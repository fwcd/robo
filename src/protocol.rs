use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

impl Default for MouseButton {
    fn default() -> Self { Self::Left }
}

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
