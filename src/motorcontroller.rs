use tokio::sync::mpsc;
use crate::motorstepper::*;
use async_trait::async_trait;
use crate::node::Node;
use tokio::time::sleep;
use std::time::Duration;

pub struct MotorController {
  pub tx   : mpsc::Sender<MotorStepCommand>,
}

impl MotorController {
  pub fn new(stepper : &Box<MotorStepper>) -> Box<MotorController> {
    Box::new(MotorController {
      tx: stepper.tx.clone(),
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
    let mut speed : f32 = 200.0;

    loop
    {
        sleep(Duration::from_millis(250)).await;
        speed = speed + 20.0;
        println!("Setting new motor speed to {}", speed);
        match self.tx.send(MotorStepCommand::Rotate(speed)).await
        {
            Err(_) => { println!("Could not send new motor speed"); }
            _ => { }
        }
    }
  }
}