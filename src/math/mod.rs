pub use ndarray::*;
pub mod vec2;

pub use vec2::Vec2;

pub fn lerp(a: f32, b: f32, mut p: f32) -> f32 {
    p = clamp(p, 0.0, 1.0);
    a * (1.0 - p) + b * p
}

pub fn min(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}

pub fn max(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}

pub fn clamp(value: f32, _min: f32, _max: f32) -> f32 {
    min(max(value, _min), _max)
}

pub fn abs(value: f32) -> f32 {
    if value < 0.0 {
        -value
    } else {
        value
    }
}
