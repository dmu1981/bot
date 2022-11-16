pub mod vec2;

pub use vec2::Vec2;

impl Vec2 {
    pub fn dot(&self, other: &Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn sqr_magnitude(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn magnitude(&self) -> f32 {
        self.sqr_magnitude().sqrt()
    }

    pub fn normalize(&self) -> Vec2 {
        let len = 1.0 / self.magnitude();
        Vec2 {
            x: self.x * len,
            y: self.y * len,
        }
    }
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
