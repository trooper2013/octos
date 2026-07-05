use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use octos_iac::{ArmRegistry, IacPacket};
use octos_storage::VectorStore;

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

    /// Returns a reference to the registry.
    pub fn get_registry(&self) -> Arc<RwLock<Vec<ArmRegistry>>> {
        Arc::clone(&self.registry)
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
pub async fn start_router_loop(
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

            if packet.intent == "PaymentApproval" {
                println!(
                    "[SYSTEM LOG] [UI ARM] [WIDGET] Payment approval terminal widget has been triggered."
                );
                // Generate a confirmation token
                let confirmation_token = "CONFIRM_TOKEN_1234";
                println!(
                    "[SYSTEM LOG] [UI ARM] [WIDGET] Generated confirmation token: {}",
                    confirmation_token
                );
                
                // Route packet back to the core pipeline (Logic Arm or back to sender)
                let confirm_packet = IacPacket {
                    goal_id: packet.goal_id,
                    packet_id: Uuid::new_v4(),
                    sender: ui_arm_id,
                    receiver: packet.sender, // route back to the original sender
                    intent: "PaymentConfirmation".to_string(),
                    latent_space_vector: None,
                    payload_json: format!(r#"{{"status": "confirmed", "token": "{}"}}"#, confirmation_token),
                };

                println!(
                    "[SYSTEM LOG] [UI ARM] [WIDGET] Emitting human confirmation token packet back to the loop..."
                );
                let _ = core_sender.send(confirm_packet).await;
            } else {
                println!(
                    "[SYSTEM LOG] [UI ARM] Rendering output to user console:\n{}",
                    packet.payload_json
                );
                println!(
                    "[SYSTEM LOG] [UI ARM] State goal successfully completed. Terminating simulator run loop."
                );
                break;
            }
        } else {
            println!(
                "[SYSTEM LOG] [CORE] [WARNING] Packet addressed to unknown arm ID: {}",
                packet.receiver
            );
        }
    }
}
