use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}
