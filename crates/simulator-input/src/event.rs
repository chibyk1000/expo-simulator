use serde::{Deserialize, Serialize};
use simulator_core::Point;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    MouseDown {
        point: Point,
        button: u32,
    },
    MouseUp {
        point: Point,
        button: u32,
    },
    MouseMove {
        point: Point,
    },
    Scroll {
        delta_x: f32,
        delta_y: f32,
    },
    TextInput {
        text: String,
    },
    KeyDown {
        key: String,
    },
}
