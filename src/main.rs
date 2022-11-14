#![allow(dead_code)]
mod math;
mod node;
mod wheelcontroller;
mod motioncontroller;
mod config;
mod perception;

use tokio::sync::broadcast;
use math::Vec2;
use motioncontroller::MoveCommand;
use node::{execute, Handles};

fn custom_ctrlc_handler(ctrlc_tx : broadcast::Sender<()>)
{
  let mut ctrlc_sent = false;
  ctrlc::set_handler(move || {
    if !ctrlc_sent
    {
      println!("Custom CTRL-C handler... drop signal sent, press again to terminate forcefully");
      ctrlc_tx.send(()).unwrap();
      ctrlc_sent = true;
    }
    else
    {
      println!("Custom CTRL-C handler... terminating forcefully");
      std::process::exit(1);
    }
  }).unwrap();
}

#[tokio::main]
async fn main() {
  // Read configuration file
  let config = config::read_from_disk().unwrap();
  
  // Create channel for CTRL-C
  let (ctrlc_tx, _) = broadcast::channel(1);

  // Create our nodes
  println!("Creating nodes....");
  let wheelcontrollers = wheelcontroller::create_all(&config, &ctrlc_tx);
  let motioncontroller = motioncontroller::create(&config, &wheelcontrollers, ctrlc_tx.subscribe()).await;

  // Handle CTRL-C
  custom_ctrlc_handler(ctrlc_tx);

  // Initialization
  println!("Initializing nodes....");
  let mut init_handles = Handles::new();
  init_handles.append(&mut wheelcontroller::init(&wheelcontrollers).await);
  init_handles.append(&mut motioncontroller::init(&motioncontroller).await);
  execute(init_handles).await;

  // Run all nodes
  println!("Running nodes....");
  let mut run_handles = Handles::new();
  run_handles.append(&mut wheelcontroller::run(&wheelcontrollers).await);
  run_handles.append(&mut motioncontroller::run(&motioncontroller).await);

  let tx_move = motioncontroller::tx_move_command(&motioncontroller).await;
  tx_move.send(MoveCommand::MoveAndAlign(
    Vec2 { x: 0.0, y: 1.0 },
    Vec2 { x: 1.0, y: 0.0 }
  )).unwrap();

  execute(run_handles).await;
  
  println!("Stopping nodes....");
  let mut stop_handles = Handles::new();
  wheelcontroller::stop(&wheelcontrollers, &mut stop_handles).await;

  execute(stop_handles).await;
}


