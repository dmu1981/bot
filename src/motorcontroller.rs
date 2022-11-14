use tokio::sync::mpsc;
use crate::motorstepper::*;
use async_trait::async_trait;
use crate::node::Node;
use tokio::time::sleep;
use std::time::Duration;

pub struct MotorController {
  pub tx   : mpsc::Sender<MotorStepCommand>,
  ctrlc_rx : tokio::sync::broadcast::Receiver<()>
}

impl MotorController {
  pub fn new(stepper : &Box<MotorStepper>,
             ctrlc_rx : tokio::sync::broadcast::Receiver<()>) -> Box<MotorController> {
    Box::new(MotorController {
      tx: stepper.tx.clone(),
      ctrlc_rx: ctrlc_rx
    })
  }
}

#[async_trait]
impl Node for MotorController 
{
  async fn init(&mut self)
  {
    println!("Motor Controller initialized");
  }
  async fn run(&mut self)
  {
    let mut speed : f32 = 150.0;

    loop
    {
        if let Ok(_) = self.ctrlc_rx.try_recv() {
          return;
        }
        
        //println!("Setting new motor speed to {}", speed);
        match self.tx.send(MotorStepCommand::Rotate(speed)).await
        {
            Err(_) => { println!("Could not send new motor speed"); }
            _ => { }
        }

        sleep(Duration::from_millis(1000)).await;
        /*speed = speed - 100.0;
        if (speed < 100.0) {
          speed = 100.0;
        }*/

    }
  }
}

