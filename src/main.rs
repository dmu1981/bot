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
  let mut init_handles = Vec::<tokio::task::JoinHandle<crate::node::NodeResult>>::new();
  for wc in &wheelcontrollers {
    wc.once(wheelcontroller::init);
  }
  init_handles.push(motioncontroller.once(motioncontroller::init));

  for result in futures::future::join_all(init_handles).await.iter() {
    result.as_ref().unwrap().as_ref().unwrap();
  }

  // Run all nodes
  println!("Running nodes....");
  let mut run_handles = Vec::<tokio::task::JoinHandle<crate::node::NodeResult>>::new();
  for wc in &wheelcontrollers {
    wc.subscribe(wheelcontroller::wheel_speed_tx(wc).await.subscribe(), wheelcontroller::set_wheel_speed);
  }
  run_handles.push(motioncontroller.subscribe(motioncontroller::rx_move_command(&motioncontroller).await, motioncontroller::move_command));

  let tx_move = motioncontroller::tx_move_command(&motioncontroller).await;
  tx_move.send(MoveCommand::MoveAndAlign(
    Vec2 { x: 0.0, y: 1.0 },
    Vec2 { x: 1.0, y: 0.0 }
  )).unwrap();

  futures::future::join_all(run_handles).await;

  println!("Stopping nodes....");
  futures::future::join_all(vec![
    wheelcontrollers[0].once(wheelcontroller::reset_pins),
  ]).await;
}


