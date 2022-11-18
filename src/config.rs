use crate::math::Vec2;
use clap::Parser;
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
    pub port: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub wheels: Vec<Wheel>,
    pub simulation: Simulation,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    config: Option<String>,

    #[arg(short, long)]
    port: Option<u32>,
}

pub fn read_from_disk() -> Result<Config, Box<dyn Error>> {
    let args = Args::parse();

    let config_path = args.config.unwrap_or_else(|| "config.yaml".to_owned());
    let string = fs::read_to_string(config_path)?;
    let mut config: Config = serde_yaml::from_str(&string)?;

    let port = args.port.unwrap_or(config.simulation.port);
    let port_str = ":".to_owned() + &port.to_string();
    config.simulation.url += &port_str;
    println!("Simulation running at {}", config.simulation.url);
    Ok(config)
}
