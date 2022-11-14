use crate::config::Config;
use crate::wheelcontroller::*;
use crate::node::*;
use tokio::sync::broadcast::{Sender, Receiver};
use crate::math::{Vec2, clamp, abs};
use crate::config::WheelExtrinsics;

#[derive(Clone, Debug)]
pub enum MoveCommand
{
  MoveAndAlign(Vec2, Vec2),
  Stop
}

struct WheelInfo {
  wheel: WheelExtrinsics,
  tx: Sender<WheelSpeed>,
  extrinsics_rx: Receiver<WheelExtrinsics>
}

pub struct MotionControllerState {
  wheels: Vec<WheelInfo>,
  tx: Sender<MoveCommand>,
}

pub type MotionControllerNode = BotNode<MotionControllerState>;

pub fn init(mut state: State<MotionControllerState>) -> DynFut<NodeResult>
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

pub fn move_command(move_command: MoveCommand, state: State<MotionControllerState>) -> DynFut<NodeResult>
{
  Box::pin(async move {
    match move_command {
      MoveCommand::Stop => {
        for wheel in &state.wheels {
          wheel.tx.send(WheelSpeed::Hold).unwrap();
        };
      },
      MoveCommand::MoveAndAlign(position, _) => {
        for wheel in &state.wheels {
          //println!("Forward is {:?}", wheel.wheel.forward);
          let v : f32 = clamp(wheel.wheel.forward.dot(&position), -1.0, 1.0);
          let speed = if abs(v) < 0.05 {
            WheelSpeed::Hold
          } else if v > 0.0 {
            WheelSpeed::Cw(v)
          } else {
            WheelSpeed::Ccw(-v)
          };
          //println!("Sending {:?}", speed);
          wheel.tx.send(speed).unwrap();
        };
      }
    }
    Ok(ThreadNext::Next)
  })
}

pub async fn tx_move_command(wc: &MotionControllerNode) -> Sender<MoveCommand> {
  wc.state.lock().await.tx.clone()
}

pub async fn rx_move_command(wc: &MotionControllerNode) -> Receiver<MoveCommand> {
  wc.state.lock().await.tx.subscribe()
}

pub async fn create(
  config: &Config,
  wheelcontrollers: &[WheelControllerNode],
  drop_tx: Receiver<()>) -> MotionControllerNode 
  {
    let (tx, _) = tokio::sync::broadcast::channel::<MoveCommand>(4);

    let mut wheels = Vec::<WheelInfo>::new();

    for (index, wheel) in config.wheels.iter().enumerate() {
      wheels.push(WheelInfo {
        wheel: WheelExtrinsics {
          pivot: wheel.pivot.clone(),
          forward: wheel.forward.clone(),
        },
        tx: crate::wheelcontroller::wheel_speed_tx(&wheelcontrollers[index]).await,
        extrinsics_rx: crate::wheelcontroller::wheel_extrinsics_tx(&wheelcontrollers[index]).await, 
      });
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
