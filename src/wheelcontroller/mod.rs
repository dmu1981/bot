//use serde::{Deserialize, Serialize};

/*
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WheelSpeed {
    Cw(f32),
    Ccw(f32),
    Hold,
}*/

#[cfg(feature = "raspberry")]
pub mod wc_gpio;

#[cfg(feature = "raspberry")]
pub use self::wc_gpio::*;

#[cfg(feature = "simulation")]
pub mod wc_sim;

#[cfg(feature = "simulation")]
pub use self::wc_sim::*;
