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

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Intercom {
    pub master: bool,
    pub port: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Genetics {
    pub pool: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub wheels: Vec<Wheel>,
    pub simulation: Simulation,
    pub intercom: Intercom,
    pub genetics: Genetics,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    config: Option<String>,

    #[arg(long)]
    simport: Option<u32>,

    #[arg(long)]
    master: Option<bool>,

    #[arg(long)]
    comport: Option<u32>,

    #[arg(long)]
    genepool: Option<String>,
}

pub fn read_from_disk() -> Result<Config, Box<dyn Error>> {
    let args = Args::parse();

    let config_path = args.config.unwrap_or_else(|| "config.yaml".to_owned());
    let string = fs::read_to_string(config_path)?;
    let mut config: Config = serde_yaml::from_str(&string)?;

    // Overwrite master mode if set via command line
    if let Some(genepool) = args.genepool {
        config.genetics.pool = genepool;
    }

    if let Some(master) = args.master {
        config.intercom.master = master;
    }

    // Overwrite master mode if set via command line
    if let Some(comport) = args.comport {
        config.intercom.port = comport;
    }

    let port = args.simport.unwrap_or(config.simulation.port);
    let port_str = ":".to_owned() + &port.to_string();
    config.simulation.url += &port_str;
    println!("Simulation running at {}", config.simulation.url);
    println!("AMQP running at {}", config.genetics.pool);
    Ok(config)
}
