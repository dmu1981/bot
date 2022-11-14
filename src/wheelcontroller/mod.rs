use serde::{Serialize, Deserialize};
use crate::math::Vec2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WheelSpeed {
  CW(f32),
  CCW(f32),
  Hold,
}


#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct WheelExtrinsics {
  pub pivot: Vec2,
  pub forward: Vec2
}

#[cfg(feature = "raspberry")]
pub mod wc_gpio;

#[cfg(feature = "raspberry")]
pub use self::wc_gpio::*;

#[cfg(feature = "simulation")]
pub mod wc_sim;

#[cfg(feature = "simulation")]
pub use self::wc_sim::*;