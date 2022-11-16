#![allow(dead_code)]
mod behavior;
mod config;
mod math;
mod motioncontroll;
mod node;
mod perception;
mod wheelcontroller;

use tokio::sync::broadcast;
//use math::Vec2;
//use motioncontroller::MoveCommand;
use node::execute_nodes;

fn custom_ctrlc_handler(ctrlc_tx: broadcast::Sender<()>) {
    let mut ctrlc_sent = false;
    ctrlc::set_handler(move || {
        if !ctrlc_sent {
            println!(
                "Custom CTRL-C handler... drop signal sent, press again to terminate forcefully"
            );
            ctrlc_tx.send(()).unwrap();
            ctrlc_sent = true;
        } else {
            println!("Custom CTRL-C handler... terminating forcefully");
            std::process::exit(1);
        }
    })
    .unwrap();
}

#[tokio::main]
async fn main() {
    // Read configuration file
    let config = config::read_from_disk().unwrap();

    // Create channel for CTRL-C
    let (ctrlc_tx, _) = broadcast::channel(1);

    // Create our nodes
    println!("Creating nodes....");

    let wheelcontrollers = wheelcontroller::create_all(&config, ctrlc_tx.clone());
    let motioncontroller =
        motioncontroll::create(&config, &wheelcontrollers.controllers, ctrlc_tx.subscribe());
    let perception = perception::create(&config, ctrlc_tx.clone());
    let behavior = behavior::create(
        perception.perception_rx.resubscribe(),
        motioncontroller.movecommand_tx.clone(),
        ctrlc_tx.clone(),
    );

    /*let tx_move = motioncontroller::tx_move_command(&motioncontroller.node).await;
    tx_move.send(MoveCommand::MoveAndAlign(
      Vec2 { x: 0.0, y: 1.0 },
      Vec2 { x: 1.0, y: 0.0 }
    )).unwrap();*/

    // Handle CTRL-C
    custom_ctrlc_handler(ctrlc_tx);

    // Execute nodes
    execute_nodes(vec![
        Box::new(wheelcontrollers),
        Box::new(motioncontroller),
        Box::new(perception),
        Box::new(behavior),
    ])
    .await;
}
