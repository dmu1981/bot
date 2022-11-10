use tokio::sync::mpsc;
use tokio::time::sleep;
use std::time::Duration;
use async_trait::async_trait;
use crate::node::Node;

use rppal::gpio::{Gpio, OutputPin};


pub enum MotorStepCommand {
  Rotate(f32),
  Stop,
}

enum MotorMode {
  CW,
  CCW,
  Static
}

pub struct MotorStepper {
  pub tx   : mpsc::Sender<MotorStepCommand>,

  //gpio : Gpio,

  pin1 : OutputPin, 
  pin2 : OutputPin, 
  pin3 : OutputPin, 
  pin4 : OutputPin,
  rx   : mpsc::Receiver<MotorStepCommand>,
  rolling_counter : u32,
  mode : MotorMode,
}

impl MotorStepper {
  pub fn new(gpio : &Gpio, pin1: u8, pin2: u8, pin3: u8, pin4: u8) -> Box<MotorStepper>
  {
      let (tx, rx) = mpsc::channel(4);
      //let gpio = Gpio::new().unwrap();

      Box::new(MotorStepper {
          pin1: gpio.get(pin1).unwrap().into_output(),
          pin2: gpio.get(pin2).unwrap().into_output(),
          pin3: gpio.get(pin3).unwrap().into_output(),
          pin4: gpio.get(pin4).unwrap().into_output(),
          tx: tx,
          rx: rx,
          rolling_counter: 0,
          mode : MotorMode::Static
      })
  }

  fn toggle_pins(&mut self, counter : u32) {
    match counter {
      0 => { self.pin1.set_high();  self.pin2.set_low();    self.pin3.set_low();    self.pin4.set_low(); }
      1 => { self.pin1.set_high();  self.pin2.set_high();   self.pin3.set_low();    self.pin4.set_low(); }
      2 => { self.pin1.set_low();   self.pin2.set_high();   self.pin3.set_low();    self.pin4.set_low(); }
      3 => { self.pin1.set_low();   self.pin2.set_high();   self.pin3.set_high();   self.pin4.set_low(); }
      4 => { self.pin1.set_low();   self.pin2.set_low();    self.pin3.set_high();   self.pin4.set_low(); }
      5 => { self.pin1.set_low();   self.pin2.set_low();    self.pin3.set_high();   self.pin4.set_high(); }
      6 => { self.pin1.set_low();   self.pin2.set_low();    self.pin3.set_low();    self.pin4.set_high(); }
      7 => { self.pin1.set_high();  self.pin2.set_low();    self.pin3.set_low();    self.pin4.set_high(); }
      _ => { panic!("MotorStepper encountered unknown counter!"); }
    }
  }

  fn stop(&mut self) {
    self.mode = MotorMode::Static;
    self.pin1.set_low();
    self.pin2.set_low();
    self.pin3.set_low();
    self.pin4.set_low();
  }
}

#[async_trait]
impl Node for MotorStepper {
  async fn init(&mut self)
  {
    println!("Initializing motor stepper");
    self.stop();
    println!("Motor Stepper initialized");
  }

  async fn run(&mut self)
  {
      let mut current_delay_ms : u64 = 200;

      loop {
          if let Ok(cmd) = self.rx.try_recv() {
              match cmd {
                  MotorStepCommand::Rotate(value) => {
                      self.mode = MotorMode::CW;
                      println!("Motor Stepper now rotating at {0}", value);
                      current_delay_ms = value as u64;
                  }
                  MotorStepCommand::Stop => {
                      self.stop();
                      println!("Motor Stepper stopped");
                      current_delay_ms = 5;
                  }
              }                
          }

          match self.mode {
            MotorMode::Static => { },
            MotorMode::CW => {
              if self.rolling_counter < 7 {
                self.rolling_counter += 1;
              }
              else 
              {
                self.rolling_counter = 0;
              }

              self.toggle_pins(self.rolling_counter);
            },
            MotorMode::CCW => {
              if self.rolling_counter > 0 {
                self.rolling_counter -= 1;
              }
              else 
              {
                self.rolling_counter = 7;
              }

              self.toggle_pins(self.rolling_counter);
            },
          }
          
          sleep(Duration::from_millis(current_delay_ms)).await;
      }        
  }
}
