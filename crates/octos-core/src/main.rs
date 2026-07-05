use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, RwLock};
use uuid::Uuid;

use octos_core::{
    ingestion::start_ingestion_daemon, start_analysis_arm, start_router_loop,
    start_storage_arm, start_ui_arm, OctosCore,
};
use octos_iac::{ArmCapability, ArmRegistry, IacPacket};
use octos_storage::{persistent_graph, KnowledgeNode, VectorStore};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let interactive = args.iter().any(|arg| arg == "--interactive" || arg == "-i");

    println!("[SYSTEM LOG] [BOOT] Initializing Octos simulator runtime...");
    if interactive {
        println!("[SYSTEM LOG] [BOOT] Running in INTERACTIVE mode. Stdin prompts enabled.");
    } else {
        println!("[SYSTEM LOG] [BOOT] Running in AUTOMATED mode. Stdin prompts simulated.");
    }

    // 1. Boot sequence - Load Vector DB from Disk
    let store_path = "C:\\octos\\octos\\vector_store.bin";
    println!("[SYSTEM LOG] [BOOT] Loading vector filesystem from disk at {}...", store_path);
    
    let mut db_store = persistent_graph::load_from_disk(store_path)
        .await
        .unwrap_or_else(|e| {
            eprintln!("[SYSTEM LOG] [BOOT] [WARNING] Failed to load from disk: {}. Initializing empty database.", e);
            VectorStore::new()
        });

    if db_store.nodes.is_empty() {
        println!("[SYSTEM LOG] [BOOT] Historical database not found. Populating default system nodes...");
        
        let mut meta1 = HashMap::new();
        meta1.insert("layer".to_string(), "user".to_string());
        meta1.insert("type".to_string(), "spreadsheet".to_string());
        db_store.insert(KnowledgeNode {
            id: Uuid::new_v4(),
            vector: vec![0.10, 0.90, 0.20, 0.40],
            content: "Q2 expense spreadsheet: Marketing flight tickets to SF cost $1200.".to_string(),
            metadata: meta1,
        });

        let mut meta2 = HashMap::new();
        meta2.insert("layer".to_string(), "audit".to_string());
        meta2.insert("type".to_string(), "audit_log".to_string());
        db_store.insert(KnowledgeNode {
            id: Uuid::new_v4(),
            vector: vec![0.85, 0.15, 0.60, 0.10],
            content: "Internal Audit: Spreadsheet raw lines show $5000 wire transfer anomaly to vendor Z.".to_string(),
            metadata: meta2,
        });

        let mut meta3 = HashMap::new();
        meta3.insert("layer".to_string(), "budget".to_string());
        meta3.insert("type".to_string(), "budget_layout".to_string());
        db_store.insert(KnowledgeNode {
            id: Uuid::new_v4(),
            vector: vec![0.30, 0.40, 0.80, 0.10],
            content: "Engineering compute budget layout: $300 AWS billing.".to_string(),
            metadata: meta3,
        });

        // Save fresh database to disk
        if let Err(e) = persistent_graph::save_to_disk(&db_store, store_path).await {
            eprintln!("[SYSTEM LOG] [BOOT] [ERROR] Failed to save initial database: {}", e);
        } else {
            println!("[SYSTEM LOG] [BOOT] Initial database persisted to disk.");
        }
    } else {
        println!("[SYSTEM LOG] [BOOT] Historical database loaded. Total nodes: {}", db_store.nodes.len());
        // Verify past inputs can be found semantically
        let verify_vector = vec![0.80, 0.10, 0.50, 0.10];
        let search_verify = db_store.search(&verify_vector, 1);
        if let Some(best_match) = search_verify.first() {
            println!(
                "[SYSTEM LOG] [BOOT] Historical semantic validation: Found best match: \"{}\"",
                best_match.content
            );
        }
    }

    let vector_store = Arc::new(RwLock::new(db_store));

    // 2. Setup channels for message routing bus
    let (core_tx, core_rx) = mpsc::channel::<IacPacket>(100);

    // 3. Initialize Core Engine
    let core = OctosCore::new(core_tx.clone());

    // 4. Generate individual channels for persistent Arms
    let (ui_tx, ui_rx) = mpsc::channel::<IacPacket>(10);
    let (storage_tx, storage_rx) = mpsc::channel::<IacPacket>(10);
    let (analysis_tx, analysis_rx) = mpsc::channel::<IacPacket>(10);

    // 5. Generate Registry for Arms
    let ui_arm_id = Uuid::new_v4();
    let ui_arm = ArmRegistry {
        arm_id: ui_arm_id,
        name: "UI Arm".to_string(),
        capabilities: vec![ArmCapability::UserInterface],
    };

    let storage_arm_id = Uuid::new_v4();
    let storage_arm = ArmRegistry {
        arm_id: storage_arm_id,
        name: "Storage Arm".to_string(),
        capabilities: vec![ArmCapability::SemanticStorage],
    };

    let analysis_arm_id = Uuid::new_v4();
    let analysis_arm = ArmRegistry {
        arm_id: analysis_arm_id,
        name: "Analysis Arm".to_string(),
        capabilities: vec![ArmCapability::CodeExecution],
    };

    // Register all Arms along with their local channels
    core.register_arm(ui_arm, ui_tx).await;
    core.register_arm(storage_arm, storage_tx).await;
    core.register_arm(analysis_arm, analysis_tx).await;

    // 6. Setup Ingestion Daemon background channel
    let (ingest_tx, ingest_rx) = mpsc::channel::<String>(100);
    let ingestion_handle = tokio::spawn(start_ingestion_daemon(ingest_rx, Arc::clone(&vector_store)));

    // 7. Setup shutdown trigger
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // 8. Spawn persistent asynchronous tasks representing the core Arms
    let storage_handle = tokio::spawn(start_storage_arm(
        storage_rx,
        Arc::clone(&vector_store),
        core_tx.clone(),
    ));

    let analysis_handle = tokio::spawn(start_analysis_arm(
        analysis_rx,
        core_tx.clone(),
        ui_arm_id,
    ));

    let ui_handle = tokio::spawn(start_ui_arm(
        ui_rx,
        core_tx.clone(),
        shutdown_tx,
        interactive,
    ));

    // Spawn central router task
    let senders_clone = core.get_senders();
    let router_handle = tokio::spawn(start_router_loop(core_rx, senders_clone));

    // 9. Inject user goal
    let goal_desc = "Analyze my local expense spreadsheets from last month and flag anomalies.";
    core.broadcast_goal(Uuid::new_v4(), goal_desc).await;

    // Log the user's interactive goal input asynchronously to disk via the Ingestion Daemon
    let _ = ingest_tx.send(goal_desc.to_string()).await;

    // Dispatch the first user packet from Analysis Arm to the Storage Arm
    let query_vector = vec![0.80, 0.10, 0.50, 0.10];
    let query_packet = IacPacket {
        goal_id: Uuid::new_v4(),
        packet_id: Uuid::new_v4(),
        sender: analysis_arm_id,
        receiver: storage_arm_id,
        intent: "SearchVectorFileSystem".to_string(),
        latent_space_vector: Some(query_vector),
        payload_json: r#"{"query": "expense spreadsheet anomaly"}"#.to_string(),
    };

    println!("[SYSTEM LOG] [CORE] Injecting first semantic search request on behalf of Analysis Arm...");
    core.route_packet(query_packet).await;

    // Wait for UI Arm shutdown trigger (end of scenario)
    let _ = shutdown_rx.await;

    // Save final vector database back to disk before terminating
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await; // allow final ingestion daemon tasks to complete
    let final_store = vector_store.read().await;
    if let Err(e) = persistent_graph::save_to_disk(&final_store, store_path).await {
        eprintln!("[SYSTEM LOG] [SHUTDOWN] [ERROR] Failed to save database to disk: {}", e);
    } else {
        println!("[SYSTEM LOG] [SHUTDOWN] Vector database persisted to disk successfully.");
    }

    // Clean up: abort remaining background tasks
    storage_handle.abort();
    analysis_handle.abort();
    ui_handle.abort();
    router_handle.abort();
    ingestion_handle.abort();

    println!("[SYSTEM LOG] [SHUTDOWN] Octos simulation execution ended successfully.");
}
