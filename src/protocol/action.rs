use serde::{Serialize, Deserialize};

use super::{Vec2, MouseButton};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    // Keyboard
    KeySequence { text: String },
    // Mouse
    MouseMoveTo { point: Vec2<i32> },
    MouseMoveBy { delta: Vec2<i32> },
    MouseDown {
        #[serde(default)]
        button: MouseButton
    },
    MouseUp {
        #[serde(default)]
        button: MouseButton
    },
    MouseClick {
        #[serde(default)]
        button: MouseButton
    },
}
