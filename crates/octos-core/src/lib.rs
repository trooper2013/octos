pub mod ui_arm;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use octos_iac::{ArmRegistry, IacPacket};
use octos_storage::VectorStore;
use ui_arm::render_dynamic_widget;

/// The central engine orchestrator for Octos.
pub struct OctosCore {
    registry: Arc<RwLock<Vec<ArmRegistry>>>,
    arm_senders: Arc<RwLock<HashMap<Uuid, mpsc::Sender<IacPacket>>>>,
    packet_tx: mpsc::Sender<IacPacket>,
}

impl OctosCore {
    /// Instantiates a new OctosCore.
    pub fn new(packet_tx: mpsc::Sender<IacPacket>) -> Self {
        Self {
            registry: Arc::new(RwLock::new(Vec::new())),
            arm_senders: Arc::new(RwLock::new(HashMap::new())),
            packet_tx,
        }
    }

    /// Returns a reference to the registry.
    pub fn get_registry(&self) -> Arc<RwLock<Vec<ArmRegistry>>> {
        Arc::clone(&self.registry)
    }

    /// Registers a capability-endowed Arm along with its routing channel.
    pub async fn register_arm(&self, arm: ArmRegistry, sender: mpsc::Sender<IacPacket>) {
        println!(
            "[SYSTEM LOG] [CORE] Registering Arm '{}' (ID: {}) with capabilities: {:?}",
            arm.name, arm.arm_id, arm.capabilities
        );
        let mut registry = self.registry.write().await;
        let mut senders = self.arm_senders.write().await;
        senders.insert(arm.arm_id, sender);
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
            "[SYSTEM LOG] [CORE] Routing packet {} from Arm {} to Arm {} (Intent: {})",
            packet.packet_id, packet.sender, packet.receiver, packet.intent
        );
        if let Err(e) = self.packet_tx.send(packet).await {
            eprintln!(
                "[SYSTEM LOG] [CORE] [ERROR] Failed to send packet onto the bus: {}",
                e
            );
        }
    }

    /// Exposes a copy of the active senders mapping.
    pub fn get_senders(&self) -> Arc<RwLock<HashMap<Uuid, mpsc::Sender<IacPacket>>>> {
        Arc::clone(&self.arm_senders)
    }
}

/// Asynchronous router execution loop that processes packets from the channel and routes to target Arms.
pub async fn start_router_loop(
    mut packet_rx: mpsc::Receiver<IacPacket>,
    arm_senders: Arc<RwLock<HashMap<Uuid, mpsc::Sender<IacPacket>>>>,
) {
    while let Some(packet) = packet_rx.recv().await {
        println!(
            "[SYSTEM LOG] [BUS] Dispatching packet ID: {} | Intent: '{}' | Receiver: {}",
            packet.packet_id, packet.intent, packet.receiver
        );
        let senders = arm_senders.read().await;
        if let Some(sender) = senders.get(&packet.receiver) {
            if let Err(e) = sender.send(packet).await {
                eprintln!(
                    "[SYSTEM LOG] [BUS] [ERROR] Failed to forward packet to target arm: {}",
                    e
                );
            }
        } else {
            eprintln!(
                "[SYSTEM LOG] [BUS] [WARNING] Destination Arm {} is not active or registered.",
                packet.receiver
            );
        }
    }
}

/// Persistent Storage Arm task processing vector DB search requests.
pub async fn start_storage_arm(
    mut rx: mpsc::Receiver<IacPacket>,
    vector_store: Arc<VectorStore>,
    core_tx: mpsc::Sender<IacPacket>,
) {
    println!("[SYSTEM LOG] [STORAGE ARM] Persistent task started.");
    while let Some(packet) = rx.recv().await {
        println!(
            "[SYSTEM LOG] [STORAGE ARM] Received request for intent: '{}'",
            packet.intent
        );
        if packet.intent == "SearchVectorFileSystem" {
            if let Some(query_vector) = &packet.latent_space_vector {
                println!(
                    "[SYSTEM LOG] [STORAGE ARM] Performing cosine similarity search..."
                );
                let search_results = vector_store.search(query_vector, 2);
                println!(
                    "[SYSTEM LOG] [STORAGE ARM] Found {} nodes:",
                    search_results.len()
                );
                for (idx, node) in search_results.iter().enumerate() {
                    println!(
                        "  -> Match #{}: '{}' (Metadata: {:?})",
                        idx + 1,
                        node.content,
                        node.metadata
                    );
                }

                let payload = serde_json::to_string(&search_results).unwrap_or_else(|_| "[]".to_string());
                let response = IacPacket {
                    goal_id: packet.goal_id,
                    packet_id: Uuid::new_v4(),
                    sender: packet.receiver,
                    receiver: packet.sender, // send back to the requester (Analysis Arm)
                    intent: "ProcessSearchResults".to_string(),
                    latent_space_vector: None,
                    payload_json: payload,
                };
                let _ = core_tx.send(response).await;
            }
        }
    }
    println!("[SYSTEM LOG] [STORAGE ARM] Persistent task terminated.");
}

/// Persistent Analysis Arm task processing search results, detecting anomalies, and coordinating approval.
pub async fn start_analysis_arm(
    mut rx: mpsc::Receiver<IacPacket>,
    core_tx: mpsc::Sender<IacPacket>,
    ui_arm_id: Uuid,
) {
    println!("[SYSTEM LOG] [ANALYSIS ARM] Persistent task started.");
    while let Some(packet) = rx.recv().await {
        println!(
            "[SYSTEM LOG] [ANALYSIS ARM] Received request for intent: '{}'",
            packet.intent
        );
        if packet.intent == "ProcessSearchResults" {
            println!(
                "[SYSTEM LOG] [ANALYSIS ARM] Parsing spreadsheet vector data..."
            );
            // Simulate anomaly detection
            println!(
                "[SYSTEM LOG] [ANALYSIS ARM] [ANOMALY DETECTED] Expense spreadsheet contains unexpected $5000 wire transfer."
            );
            println!(
                "[SYSTEM LOG] [ANALYSIS ARM] Demanding human confirmation for verification."
            );

            // Send intent to UI Arm
            let approval_packet = IacPacket {
                goal_id: packet.goal_id,
                packet_id: Uuid::new_v4(),
                sender: packet.receiver, // analysis arm id
                receiver: ui_arm_id,
                intent: "approve_payment".to_string(),
                latent_space_vector: None,
                payload_json: r#"{"amount": 5000, "description": "Vendor Z wire anomaly"}"#.to_string(),
            };
            let _ = core_tx.send(approval_packet).await;
        } else if packet.intent == "PaymentConfirmation" {
            if packet.payload_json.contains("\"status\":\"declined\"") || packet.payload_json.contains("\"status\": \"declined\"") {
                println!(
                    "[SYSTEM LOG] [ANALYSIS ARM] Payment authorization was DECLINED by the human operator."
                );
                println!(
                    "[SYSTEM LOG] [ANALYSIS ARM] Aborting expense audit analysis."
                );

                let final_packet = IacPacket {
                    goal_id: packet.goal_id,
                    packet_id: Uuid::new_v4(),
                    sender: packet.receiver,
                    receiver: packet.sender, // UI arm id
                    intent: "DisplayGoalResolution".to_string(),
                    latent_space_vector: None,
                    payload_json: r#"{"status": "cancelled", "resolution": "Analysis aborted. Anomaly was declined by human auditor."}"#.to_string(),
                };
                let _ = core_tx.send(final_packet).await;
            } else {
                println!(
                    "[SYSTEM LOG] [ANALYSIS ARM] Payment authorization received from human operator."
                );
                println!(
                    "[SYSTEM LOG] [ANALYSIS ARM] Payload: {}",
                    packet.payload_json
                );
                println!(
                    "[SYSTEM LOG] [ANALYSIS ARM] Wrapping up analysis and marking goal complete."
                );

                // Route final update back to UI Arm to show end results
                let final_packet = IacPacket {
                    goal_id: packet.goal_id,
                    packet_id: Uuid::new_v4(),
                    sender: packet.receiver,
                    receiver: packet.sender, // UI arm id
                    intent: "DisplayGoalResolution".to_string(),
                    latent_space_vector: None,
                    payload_json: r#"{"status": "completed", "resolution": "Analysis finished. Anomaly audited and approved."}"#.to_string(),
                };
                let _ = core_tx.send(final_packet).await;
            }
        }
    }
    println!("[SYSTEM LOG] [ANALYSIS ARM] Persistent task terminated.");
}

/// Persistent UI Arm task handling display rendering, widget triggering, and shutdown orchestration.
pub async fn start_ui_arm(
    mut rx: mpsc::Receiver<IacPacket>,
    core_tx: mpsc::Sender<IacPacket>,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    interactive: bool,
) {
    println!("[SYSTEM LOG] [UI ARM] Persistent task started.");
    
    let mut shutdown_tx = Some(shutdown_tx);

    while let Some(packet) = rx.recv().await {
        println!(
            "[SYSTEM LOG] [UI ARM] Received request for intent: '{}'",
            packet.intent
        );
        if packet.intent == "approve_payment" || packet.intent == "select_photo" {
            println!(
                "[SYSTEM LOG] [UI ARM] Invoking terminal dynamic widget..."
            );
            
            let result = render_dynamic_widget(&packet.intent, &packet.payload_json, interactive).await;
            
            let reply_packet = match result {
                Some(token) => IacPacket {
                    goal_id: packet.goal_id,
                    packet_id: Uuid::new_v4(),
                    sender: packet.receiver,
                    receiver: packet.sender,
                    intent: "PaymentConfirmation".to_string(),
                    latent_space_vector: None,
                    payload_json: format!(r#"{{"token": "{}", "status": "confirmed"}}"#, token),
                },
                None => IacPacket {
                    goal_id: packet.goal_id,
                    packet_id: Uuid::new_v4(),
                    sender: packet.receiver,
                    receiver: packet.sender,
                    intent: "PaymentConfirmation".to_string(),
                    latent_space_vector: None,
                    payload_json: r#"{"token": "", "status": "declined"}"#.to_string(),
                },
            };
            let _ = core_tx.send(reply_packet).await;
        } else if packet.intent == "DisplayGoalResolution" {
            println!(
                "[SYSTEM LOG] [UI ARM] Displaying Final Resolution to user terminal:\n{}",
                packet.payload_json
            );
            println!(
                "[SYSTEM LOG] [UI ARM] Triggering simulator shutdown..."
            );
            if let Some(tx) = shutdown_tx.take() {
                let _ = tx.send(());
            }
            break;
        }
    }
    println!("[SYSTEM LOG] [UI ARM] Persistent task terminated.");
}
