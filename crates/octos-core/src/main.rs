use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use octos_iac::{ArmCapability, ArmRegistry, IacPacket};
use octos_storage::{KnowledgeNode, VectorStore};

/// The central engine orchestrator for Octos.
pub struct OctosCore {
    registry: Arc<RwLock<Vec<ArmRegistry>>>,
    packet_tx: mpsc::Sender<IacPacket>,
}

impl OctosCore {
    /// Instantiates a new OctosCore.
    pub fn new(packet_tx: mpsc::Sender<IacPacket>) -> Self {
        Self {
            registry: Arc::new(RwLock::new(Vec::new())),
            packet_tx,
        }
    }

    /// Registers a capability-endowed Arm on the core bus registry.
    pub async fn register_arm(&self, arm: ArmRegistry) {
        println!(
            "[SYSTEM LOG] [CORE] Registering Arm '{}' (ID: {}) with capabilities: {:?}",
            arm.name, arm.arm_id, arm.capabilities
        );
        let mut registry = self.registry.write().await;
        registry.push(arm);
    }

    /// Broadcasts a Goal notification across system logging.
    pub async fn broadcast_goal(&self, goal_id: Uuid, description: &str) {
        println!(
            "\n[SYSTEM LOG] [CORE] ========================================"
        );
        println!(
            "[SYSTEM LOG] [CORE] BROADCASTING NEW GOAL (Goal ID: {})",
            goal_id
        );
        println!(
            "[SYSTEM LOG] [CORE] Goal Description: \"{}\"",
            description
        );
        println!(
            "[SYSTEM LOG] [CORE] ========================================\n"
        );
    }

    /// Asynchronously routes a packet through the core routing bus.
    pub async fn route_packet(&self, packet: IacPacket) {
        println!(
            "[SYSTEM LOG] [CORE] Asynchronously routing packet {} from Arm {} to Arm {} (Intent: {})",
            packet.packet_id, packet.sender, packet.receiver, packet.intent
        );
        if let Err(e) = self.packet_tx.send(packet).await {
            eprintln!(
                "[SYSTEM LOG] [CORE] [ERROR] Failed to send packet onto the bus: {}",
                e
            );
        }
    }
}

/// Asynchronous router execution loop that processes packets from the channel.
async fn start_router_loop(
    mut packet_rx: mpsc::Receiver<IacPacket>,
    vector_store: Arc<VectorStore>,
    ui_arm_id: Uuid,
    storage_arm_id: Uuid,
    logic_arm_id: Uuid,
    core_sender: mpsc::Sender<IacPacket>,
) {
    while let Some(packet) = packet_rx.recv().await {
        println!(
            "\n[SYSTEM LOG] [BUS] Dispatching packet ID: {} | Intent: '{}'",
            packet.packet_id, packet.intent
        );

        if packet.receiver == storage_arm_id {
            println!(
                "[SYSTEM LOG] [STORAGE ARM] Received request for intent: '{}'",
                packet.intent
            );
            if let Some(query_vector) = &packet.latent_space_vector {
                println!(
                    "[SYSTEM LOG] [STORAGE ARM] Executing cosine similarity search for latent vector: {:?}",
                    query_vector
                );
                let search_results = vector_store.search(query_vector, 2);
                println!(
                    "[SYSTEM LOG] [STORAGE ARM] Search completed. Found {} matching knowledge nodes:",
                    search_results.len()
                );
                for (idx, node) in search_results.iter().enumerate() {
                    println!(
                        "  -> Match #{}: Content: '{}' | ID: {} | Metadata: {:?}",
                        idx + 1,
                        node.content,
                        node.id,
                        node.metadata
                    );
                }

                // Serialize results for transmission to the Logic Arm
                let payload = serde_json::to_string(&search_results).unwrap_or_else(|_| "[]".to_string());

                let response_packet = IacPacket {
                    goal_id: packet.goal_id,
                    packet_id: Uuid::new_v4(),
                    sender: storage_arm_id,
                    receiver: logic_arm_id,
                    intent: "ProcessSearchResults".to_string(),
                    latent_space_vector: None,
                    payload_json: payload,
                };

                println!(
                    "[SYSTEM LOG] [STORAGE ARM] Emitting response packet to Logic Arm for processing..."
                );
                let _ = core_sender.send(response_packet).await;
            } else {
                println!(
                    "[SYSTEM LOG] [STORAGE ARM] [WARNING] Missing query vector in search packet."
                );
            }
        } else if packet.receiver == logic_arm_id {
            println!(
                "[SYSTEM LOG] [LOGIC ARM] Received request for intent: '{}'",
                packet.intent
            );
            println!(
                "[SYSTEM LOG] [LOGIC ARM] Serialized Payload: {}",
                packet.payload_json
            );
            println!(
                "[SYSTEM LOG] [LOGIC ARM] Executing mock logical reasoning on retrieved filesystem content..."
            );

            // Send notification back to UI Arm
            let reply_packet = IacPacket {
                goal_id: packet.goal_id,
                packet_id: Uuid::new_v4(),
                sender: logic_arm_id,
                receiver: ui_arm_id,
                intent: "DisplayGoalResolution".to_string(),
                latent_space_vector: None,
                payload_json: r#"{"status": "success", "output": "Memory management allocations verified: zero-copy capabilities successfully configured."}"#.to_string(),
            };

            println!(
                "[SYSTEM LOG] [LOGIC ARM] Emitting goal resolution response to UI Arm..."
            );
            let _ = core_sender.send(reply_packet).await;
        } else if packet.receiver == ui_arm_id {
            println!(
                "[SYSTEM LOG] [UI ARM] Received request for intent: '{}'",
                packet.intent
            );
            println!(
                "[SYSTEM LOG] [UI ARM] Rendering output to user console:\n{}",
                packet.payload_json
            );
            println!(
                "[SYSTEM LOG] [UI ARM] State goal successfully completed. Terminating simulator run loop."
            );
            break;
        } else {
            println!(
                "[SYSTEM LOG] [CORE] [WARNING] Packet addressed to unknown arm ID: {}",
                packet.receiver
            );
        }
    }
}

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
    // Latent space vector representing "memory management zero-copy capability descriptor" in 4D space
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
