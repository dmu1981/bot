use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
