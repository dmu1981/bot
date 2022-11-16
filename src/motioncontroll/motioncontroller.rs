use crate::config::Config;
use crate::config::WheelExtrinsics;
use crate::math::{abs, clamp, Vec2};
use crate::node::*;
use crate::wheelcontroller::*;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub enum MoveCommand {
    MoveAndAlign(Vec2, Vec2),
    Stop,
}

struct WheelInfo {
    wheel: WheelExtrinsics,
    tx: Sender<WheelSpeed>,
    extrinsics_rx: Receiver<WheelExtrinsics>,
}

struct MotionControllerState {
    wheels: Vec<WheelInfo>,
    drop_rx: Receiver<()>,
}

pub struct MotionControllerNode {
    pub movecommand_tx: Sender<MoveCommand>,
    drop_rx: Receiver<()>,
    state: Arc<Mutex<MotionControllerState>>,
}

fn init_node(mut state: State<MotionControllerState>) -> DynFut<NodeResult> {
    Box::pin(async move {
        let mut drop_rx = state.drop_rx.resubscribe();

        for wheel in &mut state.wheels {
            tokio::select! {
              _ = drop_rx.recv() => {
                break;
              },
              value = wheel.extrinsics_rx.recv() => {
                wheel.wheel = value.unwrap();

                println!(
                  "MotionController received extrinsics for wheel: {:?}",
                  wheel.wheel
                );
              }
            }
        }

        Ok(ThreadNext::Terminate)
    })
}

fn move_command(
    move_command: MoveCommand,
    state: State<MotionControllerState>,
) -> DynFut<NodeResult> {
    Box::pin(async move {
        match move_command {
            MoveCommand::Stop => {
                for wheel in &state.wheels {
                    wheel.tx.send(WheelSpeed::Hold).unwrap();
                }
            }
            MoveCommand::MoveAndAlign(position, _) => {
                for wheel in &state.wheels {
                    //println!("Forward is {:?}", wheel.wheel.forward);
                    let v: f32 = clamp(wheel.wheel.forward.dot(&position.normalize()), -1.0, 1.0);
                    let speed = if abs(v) < 0.05 {
                        WheelSpeed::Hold
                    } else if v > 0.0 {
                        WheelSpeed::Cw(v)
                    } else {
                        WheelSpeed::Ccw(-v)
                    };
                    //println!("Sending {:?}", speed);
                    wheel.tx.send(speed).unwrap();
                }
            }
        }
        Ok(ThreadNext::Next)
    })
}

pub fn create(
    config: &Config,
    wheelcontrollers: &[WheelControllerNode],
    drop_rx: Receiver<()>,
) -> MotionControllerNode {
    let (tx, _) = tokio::sync::broadcast::channel::<MoveCommand>(4);

    let mut wheels = Vec::<WheelInfo>::new();

    for (index, wheel) in config.wheels.iter().enumerate() {
        wheels.push(WheelInfo {
            wheel: WheelExtrinsics {
                pivot: wheel.pivot.clone(),
                forward: wheel.forward.clone(),
            },
            tx: wheelcontrollers[index].wheelspeed_tx.clone(),
            extrinsics_rx: wheelcontrollers[index].extrinsics_rx.resubscribe(),
        });
    }

    MotionControllerNode {
        drop_rx: drop_rx.resubscribe(),
        movecommand_tx: tx,
        state: Arc::new(Mutex::new(MotionControllerState {
            wheels,
            drop_rx: drop_rx.resubscribe(),
        })),
    }
}

impl Node<MotionControllerState> for MotionControllerNode {
    fn get_state_handle(&self) -> Arc<Mutex<MotionControllerState>> {
        self.state.clone()
    }

    fn get_drop_rx(&self) -> Receiver<()> {
        self.drop_rx.resubscribe()
    }
}

#[async_trait]
impl Executor for MotionControllerNode {
    async fn init(&self) -> Handles {
        vec![self.once(init_node)]
    }

    async fn run(&self) -> Handles {
        vec![self.subscribe(self.movecommand_tx.subscribe(), move_command)]
    }

    async fn stop(&self) -> Handles {
        vec![]
    }
}
