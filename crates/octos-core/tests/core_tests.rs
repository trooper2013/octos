use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use octos_core::{start_router_loop, start_ui_arm, OctosCore};
use octos_iac::{ArmCapability, ArmRegistry, IacPacket};

#[tokio::test]
async fn test_async_arm_registration() {
    let (tx, _rx) = mpsc::channel::<IacPacket>(10);
    let core = OctosCore::new(tx);

    let (arm_tx1, _arm_rx1) = mpsc::channel::<IacPacket>(10);
    let (arm_tx2, _arm_rx2) = mpsc::channel::<IacPacket>(10);

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

    core.register_arm(arm1, arm_tx1).await;
    core.register_arm(arm2, arm_tx2).await;

    let registry = core.get_registry();
    let registry_read = registry.read().await;
    assert_eq!(registry_read.len(), 2);
    assert_eq!(registry_read[0].name, "TestArm1");
    assert_eq!(registry_read[1].name, "TestArm2");
    
    let senders = core.get_senders();
    let senders_read = senders.read().await;
    assert_eq!(senders_read.len(), 2);
}

#[tokio::test]
async fn test_packet_routing_delivery() {
    let (core_tx, core_rx) = mpsc::channel::<IacPacket>(10);
    let core = OctosCore::new(core_tx);

    let (arm_tx, mut arm_rx) = mpsc::channel::<IacPacket>(10);
    let arm_id = Uuid::new_v4();
    let arm = ArmRegistry {
        arm_id,
        name: "TestReceiverArm".to_string(),
        capabilities: vec![ArmCapability::UserInterface],
    };

    core.register_arm(arm, arm_tx).await;

    let goal_id = Uuid::new_v4();
    let packet_id = Uuid::new_v4();
    let sender = Uuid::new_v4();

    let packet = IacPacket {
        goal_id,
        packet_id,
        sender,
        receiver: arm_id,
        intent: "DeliverTest".to_string(),
        latent_space_vector: None,
        payload_json: r#"{"data": 123}"#.to_string(),
    };

    // Start router loop
    let senders_clone = core.get_senders();
    let router_handle = tokio::spawn(start_router_loop(core_rx, senders_clone));

    core.route_packet(packet).await;

    let received = arm_rx.recv().await.expect("Failed to receive packet at arm channel");
    assert_eq!(received.packet_id, packet_id);
    assert_eq!(received.receiver, arm_id);

    router_handle.abort();
}

#[tokio::test]
async fn test_ui_arm_dynamic_widget_trigger() {
    let (ui_tx, ui_rx) = mpsc::channel::<IacPacket>(10);
    let (core_tx, mut core_rx) = mpsc::channel::<IacPacket>(10);
    let (shutdown_tx, _shutdown_rx) = oneshot::channel::<()>();

    let goal_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    let ui_arm_id = Uuid::new_v4();

    // Create the payment approval packet directed at the UI Arm
    let payment_packet = IacPacket {
        goal_id,
        packet_id: Uuid::new_v4(),
        sender: sender_id,
        receiver: ui_arm_id,
        intent: "approve_payment".to_string(),
        latent_space_vector: None,
        payload_json: r#"{"amount": 5000, "description": "Vendor Z wire anomaly"}"#.to_string(),
    };

    // Send it directly to UI arm's channel
    ui_tx.send(payment_packet).await.unwrap();

    // Spawn the persistent UI arm task
    let ui_handle = tokio::spawn(start_ui_arm(
        ui_rx,
        core_tx,
        shutdown_tx,
    ));

    // The UI arm should process it and send a PaymentConfirmation packet back to the core bus channel
    let response = core_rx.recv().await.expect("Failed to receive packet from UI arm");
    assert_eq!(response.goal_id, goal_id);
    assert_eq!(response.intent, "PaymentConfirmation");
    assert_eq!(response.receiver, sender_id);
    assert!(response.payload_json.contains("TOKEN-VERIFY-"));

    // Cleanup
    drop(ui_tx);
    let _ = ui_handle.await;
}
