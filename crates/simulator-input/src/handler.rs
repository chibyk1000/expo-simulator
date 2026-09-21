use crate::event::InputEvent;
use simulator_bridge::{BridgeChannel, BridgeResponse};
use simulator_core::{ComponentType, Node, NodeId, Point, Rect};
use std::collections::HashMap;

pub struct InputHandler {
    bridge: BridgeChannel,
    active_touch_target: Option<NodeId>,
    focused_input_target: Option<NodeId>,
}

impl InputHandler {
    pub fn new(bridge: BridgeChannel) -> Self {
        Self {
            bridge,
            active_touch_target: None,
            focused_input_target: None,
        }
    }

    pub fn focused_node(&self) -> Option<NodeId> {
        self.focused_input_target
    }

    pub fn active_touch_target(&self) -> Option<NodeId> {
        self.active_touch_target
    }

    pub fn handle_event(
        &mut self,
        event: InputEvent,
        nodes: &HashMap<NodeId, Node>,
        root_nodes: &[NodeId],
        device_screen_rect: Rect,
    ) {
        match event {
            InputEvent::MouseDown { point, .. } => {
                // Check if click is inside the mobile device screen
                if !device_screen_rect.contains(point) {
                    return;
                }

                // Transform point to device-relative coordinates
                let local_point = Point::new(
                    point.x - device_screen_rect.x,
                    point.y - device_screen_rect.y,
                );

                if let Some(target_id) = self.hit_test(local_point, nodes, root_nodes) {
                    self.active_touch_target = Some(target_id);

                    // Check if it is a TextInput
                    if let Some(node) = nodes.get(&target_id) {
                        if node.component_type == ComponentType::TextInput {
                            self.focused_input_target = Some(target_id);
                        } else if self.focused_input_target.is_some() {
                            // Clicked outside focused input
                            self.focused_input_target = None;
                        }
                    }

                    // Dispatch press / touchStart to JS
                    let _ = self.bridge.send_to_js(BridgeResponse::DispatchEvent {
                        target_id: target_id.0,
                        event_name: "press".to_string(),
                        payload: serde_json::json!({
                            "locationX": local_point.x,
                            "locationY": local_point.y,
                        }),
                    });

                    log::info!("Dispatched press to {:?}", target_id);
                }
            }

            InputEvent::MouseUp { point, .. } => {
                if let Some(target_id) = self.active_touch_target.take() {
                    let local_point = Point::new(
                        point.x - device_screen_rect.x,
                        point.y - device_screen_rect.y,
                    );
                    let _ = self.bridge.send_to_js(BridgeResponse::DispatchEvent {
                        target_id: target_id.0,
                        event_name: "pressOut".to_string(),
                        payload: serde_json::json!({
                            "locationX": local_point.x,
                            "locationY": local_point.y,
                        }),
                    });
                }
            }

            InputEvent::TextInput { text } => {
                if let Some(target_id) = self.focused_input_target {
                    let _ = self.bridge.send_to_js(BridgeResponse::DispatchEvent {
                        target_id: target_id.0,
                        event_name: "changeText".to_string(),
                        payload: serde_json::json!({ "text": text }),
                    });
                }
            }

            InputEvent::KeyDown { key } => {
                if key == "Backspace" {
                    if let Some(target_id) = self.focused_input_target {
                        let _ = self.bridge.send_to_js(BridgeResponse::DispatchEvent {
                            target_id: target_id.0,
                            event_name: "backspace".to_string(),
                            payload: serde_json::json!({}),
                        });
                    }
                }
            }

            InputEvent::Scroll { delta_y, .. } => {
                // Find top-level or scrollable container
                for &root_id in root_nodes {
                    if let Some(node) = nodes.get(&root_id) {
                        if node.component_type == ComponentType::ScrollView {
                            let _ = self.bridge.send_to_js(BridgeResponse::DispatchEvent {
                                target_id: root_id.0,
                                event_name: "scroll".to_string(),
                                payload: serde_json::json!({ "deltaY": delta_y }),
                            });
                            break;
                        }
                    }
                }
            }

            _ => {}
        }
    }

    fn hit_test(
        &self,
        point: Point,
        nodes: &HashMap<NodeId, Node>,
        root_nodes: &[NodeId],
    ) -> Option<NodeId> {
        for &root_id in root_nodes.iter().rev() {
            if let Some(hit) = self.hit_test_node(point, root_id, nodes, Point::ZERO) {
                return Some(hit);
            }
        }
        None
    }

    fn hit_test_node(
        &self,
        point: Point,
        node_id: NodeId,
        nodes: &HashMap<NodeId, Node>,
        parent_offset: Point,
    ) -> Option<NodeId> {
        let node = nodes.get(&node_id)?;
        let abs_x = parent_offset.x + node.layout.x;
        let abs_y = parent_offset.y + node.layout.y;
        let bounds = Rect::new(abs_x, abs_y, node.layout.width, node.layout.height);

        if !bounds.contains(point) {
            return None;
        }

        // Test children in reverse order (topmost first)
        let child_offset = Point::new(abs_x, abs_y);
        for &child_id in node.children.iter().rev() {
            if let Some(hit) = self.hit_test_node(point, child_id, nodes, child_offset) {
                return Some(hit);
            }
        }

        // Return self if hit
        Some(node_id)
    }
}
