#![allow(dead_code)]
mod behavior;
mod config;
mod intercom;
mod kicker;
mod manager;
mod math;
mod motioncontroll;
mod node;
mod perception;
mod wheelcontroller;

use node::execute_nodes;
use tokio::sync::broadcast;

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
    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
    // Read configuration file
    let config = config::read_from_disk().unwrap();

    // Create channel for CTRL-C
    let (ctrlc_tx, _) = broadcast::channel(1);

    // Create our nodes
    println!("Creating nodes....");

    let manager = manager::create(&config, ctrlc_tx.clone());
    let intercom = intercom::create(&config, ctrlc_tx.clone());

    let wheelcontrollers = wheelcontroller::create_all(
        &config,
        manager.bot_spawned_rx.resubscribe(),
        ctrlc_tx.clone(),
    );
    let motioncontroller =
        motioncontroll::create(&config, &wheelcontrollers.controllers, ctrlc_tx.subscribe());
    let perception =
        perception::create(&config, ctrlc_tx.clone(), intercom.send_message_tx.clone());
    let kicker = kicker::create(&config, ctrlc_tx.clone());
    
    let wheelspeed_tx_vec = vec![
      wheelcontrollers.controllers[0].wheelspeed_tx.clone(),
      wheelcontrollers.controllers[1].wheelspeed_tx.clone(),
      wheelcontrollers.controllers[2].wheelspeed_tx.clone(),
      wheelcontrollers.controllers[3].wheelspeed_tx.clone(),
    ];

    let behavior = behavior::create(
        &config,
        perception.perception_rx.resubscribe(),
        motioncontroller.movecommand_tx.clone(),
        kicker.kick_tx.clone(),
        wheelspeed_tx_vec,
        manager.reset_sim_tx.clone(),
        ctrlc_tx.clone(),
    );

    // Handle CTRL-C
    custom_ctrlc_handler(ctrlc_tx);

    // Execute nodes
    execute_nodes(vec![
        Box::new(wheelcontrollers),
        Box::new(motioncontroller),
        Box::new(perception),
        Box::new(behavior),
        Box::new(kicker),
        Box::new(manager),
        Box::new(intercom),
    ])
    .await;
}
