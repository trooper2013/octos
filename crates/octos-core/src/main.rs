use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use octos_core::{start_router_loop, OctosCore};
use octos_iac::{ArmCapability, ArmRegistry, IacPacket};
use octos_storage::{KnowledgeNode, VectorStore};

#[tokio::main]
async fn main() {
    println!("[SYSTEM LOG] [BOOT] Initializing Octos simulator runtime...");

    // 1. Populate Vector File System
    let mut vector_store = VectorStore::new();

    let mut meta1 = HashMap::new();
    meta1.insert("layer".to_string(), "kernel".to_string());
    meta1.insert("module".to_string(), "memory".to_string());
    vector_store.insert(KnowledgeNode {
        id: Uuid::new_v4(),
        vector: vec![0.75, 0.15, 0.45, 0.10],
        content: "Memory management in Octos is handled via zero-copy capability descriptors.".to_string(),
        metadata: meta1,
    });

    let mut meta2 = HashMap::new();
    meta2.insert("layer".to_string(), "networking".to_string());
    meta2.insert("module".to_string(), "iac".to_string());
    vector_store.insert(KnowledgeNode {
        id: Uuid::new_v4(),
        vector: vec![0.20, 0.80, 0.10, 0.60],
        content: "The Inter-Arm Communication (IAC) protocol uses serialized JSON payloads over virtual ring buses.".to_string(),
        metadata: meta2,
    });

    let mut meta3 = HashMap::new();
    meta3.insert("layer".to_string(), "boot".to_string());
    meta3.insert("module".to_string(), "init".to_string());
    vector_store.insert(KnowledgeNode {
        id: Uuid::new_v4(),
        vector: vec![0.40, 0.30, 0.70, 0.20],
        content: "Octos boots directly into a Rust bare-metal runtime, skipping standard grub configurations.".to_string(),
        metadata: meta3,
    });

    let vector_store = Arc::new(vector_store);

    // 2. Setup channels for message routing bus
    let (packet_tx, packet_rx) = mpsc::channel::<IacPacket>(100);

    // 3. Initialize Core Engine
    let core = OctosCore::new(packet_tx.clone());

    // 4. Generate Registry for Arms
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

    let logic_arm_id = Uuid::new_v4();
    let logic_arm = ArmRegistry {
        arm_id: logic_arm_id,
        name: "Logic Arm".to_string(),
        capabilities: vec![ArmCapability::CodeExecution],
    };

    // Register all Arms
    core.register_arm(ui_arm).await;
    core.register_arm(storage_arm).await;
    core.register_arm(logic_arm).await;

    // 5. Start router execution thread
    let core_tx_clone = packet_tx.clone();
    let router_handle = tokio::spawn(async move {
        start_router_loop(
            packet_rx,
            vector_store,
            ui_arm_id,
            storage_arm_id,
            logic_arm_id,
            core_tx_clone,
        )
        .await;
    });

    // 6. Trigger Simulated User Semantic Query Goal
    let goal_id = Uuid::new_v4();
    core.broadcast_goal(
        goal_id,
        "Analyze Octos memory capability systems and print findings to the UI console.",
    )
    .await;

    // Dispatch the first user packet from UI Arm to the Storage Arm
    let query_vector = vec![0.80, 0.10, 0.50, 0.15];
    let query_packet = IacPacket {
        goal_id,
        packet_id: Uuid::new_v4(),
        sender: ui_arm_id,
        receiver: storage_arm_id,
        intent: "SearchVectorFileSystem".to_string(),
        latent_space_vector: Some(query_vector),
        payload_json: r#"{"query": "zero-copy memory capability"}"#.to_string(),
    };

    println!("[SYSTEM LOG] [UI ARM] Dispensing user query packet to the Core Bus...");
    core.route_packet(query_packet).await;

    // Wait for the simulator router to finish processing goal steps
    let _ = router_handle.await;
    println!("\n[SYSTEM LOG] [SHUTDOWN] Octos simulation execution ended successfully.");
}
