use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Enum describing capabilities of an Arm (subsystem) in the Octos architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArmCapability {
    SemanticStorage,
    CodeExecution,
    WebRetrieval,
    UserInterface,
}

/// Registry entry containing information about a registered Arm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmRegistry {
    pub arm_id: Uuid,
    pub name: String,
    pub capabilities: Vec<ArmCapability>,
}

/// Inter-Arm Communication Packet representing a message in the Octos network bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IacPacket {
    pub goal_id: Uuid,
    pub packet_id: Uuid,
    pub sender: Uuid,
    pub receiver: Uuid,
    pub intent: String,
    pub latent_space_vector: Option<Vec<f32>>,
    pub payload_json: String,
}
