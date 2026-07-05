use uuid::Uuid;
use octos_iac::{ArmCapability, ArmRegistry, IacPacket};

#[test]
fn test_iac_packet_serialization() {
    let goal_id = Uuid::new_v4();
    let packet_id = Uuid::new_v4();
    let sender = Uuid::new_v4();
    let receiver = Uuid::new_v4();
    
    let packet = IacPacket {
        goal_id,
        packet_id,
        sender,
        receiver,
        intent: "TestIntent".to_string(),
        latent_space_vector: Some(vec![0.1, 0.2, 0.3, 0.4]),
        payload_json: r#"{"hello": "world"}"#.to_string(),
    };

    let serialized = serde_json::to_string(&packet).expect("Failed to serialize");
    let deserialized: IacPacket = serde_json::from_str(&serialized).expect("Failed to deserialize");

    assert_eq!(deserialized.goal_id, packet.goal_id);
    assert_eq!(deserialized.packet_id, packet.packet_id);
    assert_eq!(deserialized.sender, packet.sender);
    assert_eq!(deserialized.receiver, packet.receiver);
    assert_eq!(deserialized.intent, packet.intent);
    assert_eq!(deserialized.latent_space_vector, packet.latent_space_vector);
    assert_eq!(deserialized.payload_json, packet.payload_json);
}

#[test]
fn test_arm_registry_capabilities() {
    let arm_id = Uuid::new_v4();
    let name = "StorageArm".to_string();
    let capabilities = vec![ArmCapability::SemanticStorage, ArmCapability::UserInterface];

    let registry = ArmRegistry {
        arm_id,
        name,
        capabilities,
    };

    assert!(registry.capabilities.contains(&ArmCapability::SemanticStorage));
    assert!(registry.capabilities.contains(&ArmCapability::UserInterface));
    assert!(!registry.capabilities.contains(&ArmCapability::CodeExecution));
}
