use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use octos_core::{
    start_analysis_arm, start_router_loop, start_storage_arm, start_ui_arm, OctosCore,
};
use octos_iac::{ArmCapability, ArmRegistry, IacPacket};
use octos_storage::{KnowledgeNode, VectorStore};

#[tokio::main]
async fn main() {
    // Check if the user requested interactive mode
    let args: Vec<String> = std::env::args().collect();
    let interactive = args.iter().any(|arg| arg == "--interactive" || arg == "-i");

    println!("[SYSTEM LOG] [BOOT] Initializing Octos simulator runtime...");
    if interactive {
        println!("[SYSTEM LOG] [BOOT] Running in INTERACTIVE mode. Stdin prompts enabled.");
    } else {
        println!("[SYSTEM LOG] [BOOT] Running in AUTOMATED mode. Stdin prompts simulated.");
    }

    // 1. Populate Vector File System
    let mut vector_store = VectorStore::new();

    let mut meta1 = HashMap::new();
    meta1.insert("layer".to_string(), "user".to_string());
    meta1.insert("type".to_string(), "spreadsheet".to_string());
    vector_store.insert(KnowledgeNode {
        id: Uuid::new_v4(),
        vector: vec![0.10, 0.90, 0.20, 0.40],
        content: "Q2 expense spreadsheet: Marketing flight tickets to SF cost $1200.".to_string(),
        metadata: meta1,
    });

    let mut meta2 = HashMap::new();
    meta2.insert("layer".to_string(), "audit".to_string());
    meta2.insert("type".to_string(), "audit_log".to_string());
    vector_store.insert(KnowledgeNode {
        id: Uuid::new_v4(),
        vector: vec![0.85, 0.15, 0.60, 0.10],
        content: "Internal Audit: Spreadsheet raw lines show $5000 wire transfer anomaly to vendor Z.".to_string(),
        metadata: meta2,
    });

    let mut meta3 = HashMap::new();
    meta3.insert("layer".to_string(), "budget".to_string());
    meta3.insert("type".to_string(), "budget_layout".to_string());
    vector_store.insert(KnowledgeNode {
        id: Uuid::new_v4(),
        vector: vec![0.30, 0.40, 0.80, 0.10],
        content: "Engineering compute budget layout: $300 AWS billing.".to_string(),
        metadata: meta3,
    });

    let vector_store = Arc::new(vector_store);

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

    // 6. Setup shutdown trigger
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // 7. Spawn persistent asynchronous tasks representing the core Arms
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

    // 8. Inject user goal
    let goal_id = Uuid::new_v4();
    core.broadcast_goal(
        goal_id,
        "Analyze my local expense spreadsheets from last month and flag anomalies.",
    )
    .await;

    // Dispatch the first user packet from Analysis Arm to the Storage Arm
    let query_vector = vec![0.80, 0.10, 0.50, 0.10];
    let query_packet = IacPacket {
        goal_id,
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

    // Clean up: abort the remaining persistent tasks
    storage_handle.abort();
    analysis_handle.abort();
    ui_handle.abort();
    router_handle.abort();

    println!("\n[SYSTEM LOG] [SHUTDOWN] Octos simulation execution ended successfully.");
}
