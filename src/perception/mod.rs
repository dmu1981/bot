use crate::math::Vec2;

#[derive(Debug, Clone)]
pub struct Measurement {
    pub position: Option<Vec2>, // Position is optional as it could potentially not be measured
                                // Frankly, this only makes sense when we also introduce timestamps for each measurement
}

#[derive(Debug, Clone)]
pub struct PerceptionMessage {
    pub ball: Measurement,
    pub own_goal: Measurement,
    pub target_goal: Measurement,
    pub boundary: Measurement,
}

#[cfg(feature = "simulation")]
pub mod simulation;

#[cfg(feature = "simulation")]
pub use self::simulation::*;
