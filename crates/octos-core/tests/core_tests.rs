use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use octos_core::{start_router_loop, OctosCore};
use octos_iac::{ArmCapability, ArmRegistry, IacPacket};
use octos_storage::VectorStore;

#[tokio::test]
async fn test_async_arm_registration() {
    let (tx, _rx) = mpsc::channel::<IacPacket>(10);
    let core = OctosCore::new(tx);

    let arm1 = ArmRegistry {
        arm_id: Uuid::new_v4(),
        name: "TestArm1".to_string(),
        capabilities: vec![ArmCapability::CodeExecution],
    };
    let arm2 = ArmRegistry {
        arm_id: Uuid::new_v4(),
        name: "TestArm2".to_string(),
        capabilities: vec![ArmCapability::SemanticStorage],
    };

    core.register_arm(arm1).await;
    core.register_arm(arm2).await;

    let registry = core.get_registry();
    let registry_read = registry.read().await;
    assert_eq!(registry_read.len(), 2);
    assert_eq!(registry_read[0].name, "TestArm1");
    assert_eq!(registry_read[1].name, "TestArm2");
}

#[tokio::test]
async fn test_packet_routing_delivery() {
    let (tx, mut rx) = mpsc::channel::<IacPacket>(10);
    let core = OctosCore::new(tx);

    let goal_id = Uuid::new_v4();
    let packet_id = Uuid::new_v4();
    let sender = Uuid::new_v4();
    let receiver = Uuid::new_v4();

    let packet = IacPacket {
        goal_id,
        packet_id,
        sender,
        receiver,
        intent: "DeliverTest".to_string(),
        latent_space_vector: None,
        payload_json: r#"{"data": 123}"#.to_string(),
    };

    core.route_packet(packet).await;

    let received = rx.recv().await.expect("Failed to receive packet");
    assert_eq!(received.packet_id, packet_id);
    assert_eq!(received.receiver, receiver);
}

#[tokio::test]
async fn test_ui_arm_dynamic_widget_trigger() {
    let vector_store = Arc::new(VectorStore::new());
    
    // Create channels: one for the router to read from, and one for the router to write to
    let (packet_tx, packet_rx) = mpsc::channel::<IacPacket>(10);
    let (test_tx, mut test_rx) = mpsc::channel::<IacPacket>(10);
    
    let goal_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    let ui_arm_id = Uuid::new_v4();
    let storage_arm_id = Uuid::new_v4();
    let logic_arm_id = Uuid::new_v4();

    // Create the payment approval packet directed at the UI Arm
    let payment_packet = IacPacket {
        goal_id,
        packet_id: Uuid::new_v4(),
        sender: sender_id,
        receiver: ui_arm_id,
        intent: "PaymentApproval".to_string(),
        latent_space_vector: None,
        payload_json: r#"{"amount": 500, "currency": "USD"}"#.to_string(),
    };

    // Send the payment packet into the routing queue
    packet_tx.send(payment_packet).await.unwrap();

    // Run the router loop in a background task
    let router_handle = tokio::spawn(async move {
        start_router_loop(
            packet_rx,
            vector_store,
            ui_arm_id,
            storage_arm_id,
            logic_arm_id,
            test_tx,
        )
        .await;
    });

    // We expect the router to process the PaymentApproval packet and send a PaymentConfirmation packet back to test_rx
    let confirmation_packet = test_rx.recv().await.expect("Did not receive confirmation packet");
    
    assert_eq!(confirmation_packet.goal_id, goal_id);
    assert_eq!(confirmation_packet.intent, "PaymentConfirmation");
    assert_eq!(confirmation_packet.receiver, sender_id);
    assert!(confirmation_packet.payload_json.contains("CONFIRM_TOKEN_1234"));

    // Clean up: drop packet_tx to close the loop channel so start_router_loop exits
    drop(packet_tx);
    let _ = router_handle.await;
}
