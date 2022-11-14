use crate::config::{Config, Wheel};
use crate::wheelcontroller::*;
use crate::node::*;
use tokio::sync::broadcast;
use crate::math::{Vec2, clamp, abs};

#[derive(Clone)]
pub enum MoveCommand
{
  MoveAndAlign(Vec2, Vec2),
  Stop
}

struct WheelInfo {
  wheel: WheelExtrinsics,
  tx: broadcast::Sender<WheelSpeed>,
  extrinsics_rx: broadcast::Receiver<WheelExtrinsics>
}

pub struct MotionControllerState {
  wheels: Vec<WheelInfo>,
  tx: broadcast::Sender<MoveCommand>,
}

pub type MotionControllerNode = BotNode<MotionControllerState>;

pub fn init(mut state: tokio::sync::MutexGuard<'_, MotionControllerState>) -> DynFut<'_, NodeResult>
{
  Box::pin(async move {
    for wheel in &mut state.wheels {
      let wheel_extrinsic = wheel.extrinsics_rx.recv().await.unwrap();
      //println!("MotionController received extrinsics for wheel: {:?}", wheel_extrinsic);
      wheel.wheel = wheel_extrinsic;
    }
    //state.wheels.iter
    Ok(ThreadNext::Terminate)
  })
}

pub fn move_command(move_command: MoveCommand, mut state: tokio::sync::MutexGuard<'_, MotionControllerState>) -> DynFut<'_, NodeResult>
{
  Box::pin(async move {
    match move_command {
      MoveCommand::Stop => {
        for wheel in &state.wheels {
          wheel.tx.send(WheelSpeed::Hold).unwrap();
        };
      },
      MoveCommand::MoveAndAlign(position, orientation) => {
        for wheel in &state.wheels {
          //println!("Forward is {:?}", wheel.wheel.forward);
          let v : f32 = clamp(wheel.wheel.forward.dot(&position), -1.0, 1.0);
          let speed = if abs(v) < 0.05 {
            WheelSpeed::Hold
          } else if v > 0.0 {
            WheelSpeed::CW(v)
          } else {
            WheelSpeed::CCW(-v)
          };
          //println!("Sending {:?}", speed);
          wheel.tx.send(speed).unwrap();
        };
      }
    }
    Ok(ThreadNext::Next)
  })
}

pub async fn tx_move_command(wc: &MotionControllerNode) -> tokio::sync::broadcast::Sender<MoveCommand> {
  wc.state.lock().await.tx.clone()
}

pub async fn rx_move_command(wc: &MotionControllerNode) -> tokio::sync::broadcast::Receiver<MoveCommand> {
  wc.state.lock().await.tx.subscribe()
}

pub async fn create(
  config: &Config,
  wheelcontrollers: &Vec<WheelControllerNode>,
  drop_tx: broadcast::Receiver<()>) -> MotionControllerNode 
  {
    let (tx, rx) = tokio::sync::broadcast::channel::<MoveCommand>(4);

    let mut wheels = Vec::<WheelInfo>::new();

    let mut index = 0;
    for wheel in &config.wheels {
      wheels.push(WheelInfo {
        wheel: WheelExtrinsics {
          pivot: wheel.pivot.clone(),
          forward: wheel.forward.clone(),
        },
        tx: crate::wheelcontroller::wheel_speed_tx(&wheelcontrollers[index]).await,
        extrinsics_rx: crate::wheelcontroller::wheel_extrinsics_tx(&wheelcontrollers[index]).await, 
      });

      index = index + 1;
    }

    MotionControllerNode::new(
      "Motion Controller".to_string(), 
      drop_tx, 
      MotionControllerState {
        wheels,
        tx
      }
    )
  }
