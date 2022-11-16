use crate::math::Vec2;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct WheelExtrinsics {
    pub pivot: Vec2,
    pub forward: Vec2,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Wheel {
    pub name: String,
    pub gpiopins: Vec<u8>,
    pub pivot: Vec2,
    pub forward: Vec2,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Simulation {
    pub url: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub wheels: Vec<Wheel>,
    pub simulation: Simulation,
}

pub fn read_from_disk() -> Result<Config, Box<dyn Error>> {
    let string = fs::read_to_string("config.yaml")?;
    Ok(serde_yaml::from_str(&string)?)
}
