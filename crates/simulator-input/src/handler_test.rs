use crate::{InputEvent, InputHandler};
use simulator_bridge::{BridgeChannel, BridgeResponse};
use simulator_core::{ComponentType, Node, NodeId, Point, Rect};
use std::collections::HashMap;

#[test]
fn test_hit_test_and_event_dispatch() {
    let bridge = BridgeChannel::new();
    let mut handler = InputHandler::new(bridge.clone());

    let mut nodes = HashMap::new();
    let mut button = Node::new(NodeId(4), ComponentType::Pressable);
    button.layout = Rect::new(24.0, 150.0, 345.0, 52.0);
    nodes.insert(NodeId(4), button);

    let screen_rect = Rect::new(73.5, 60.0, 393.0, 881.0);

    // Click inside the button
    let click_point = Point::new(screen_rect.x + 50.0, screen_rect.y + 160.0);
    handler.handle_event(
        InputEvent::MouseDown {
            point: click_point,
            button: 0,
        },
        &nodes,
        &[NodeId(4)],
        screen_rect,
    );

    assert_eq!(handler.active_touch_target(), Some(NodeId(4)));

    // Verify message received on the JS side of the bridge
    let rt = tokio::runtime::Runtime::new().unwrap();
    let received = rt.block_on(bridge.recv_from_host());

    assert!(matches!(
        received,
        Some(BridgeResponse::DispatchEvent {
            target_id: 4,
            ref event_name,
            ..
        }) if event_name == "press"
    ));
}
