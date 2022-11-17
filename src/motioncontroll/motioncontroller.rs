use crate::config::Config;
use crate::config::WheelExtrinsics;
use crate::math::*;
use crate::math::{clamp, Vec2};
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
    tx: Sender<f32>,
    extrinsics_rx: Receiver<WheelExtrinsics>,
    speed: f32,
}

struct MotionControllerState {
    wheels: Vec<WheelInfo>,
    drop_rx: Receiver<()>,
    rotate_momentum: f32,
    velocity_momentum: Vec2,
    last_change: std::time::Instant,
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
    mut state: State<MotionControllerState>,
) -> DynFut<NodeResult> {
    Box::pin(async move {
        match move_command {
            MoveCommand::Stop => {
                for wheel in &state.wheels {
                    wheel.tx.send(0.0).unwrap();
                }
            }
            MoveCommand::MoveAndAlign(position, orientation) => {
                let mut pos_norm = position.normalize();
                let ori_norm = orientation.normalize();

                let elapsed_s = state.last_change.elapsed().as_secs_f32();

                pos_norm -= state.velocity_momentum;

                const MAGIC_DEGREE_CONSTANT: f32 = 8.0; // Magic constant describing how quickly robot rotates based on accumulated momentum
                const MAGIC_VELOCITY_CONSTANT: f32 = 0.5; // Magic constant describing how quickly velocity accumulates
                let angle = state.rotate_momentum * MAGIC_DEGREE_CONSTANT * elapsed_s;
                let c = angle.cos();
                let s = angle.sin();
                let x = state.velocity_momentum.x * c - state.velocity_momentum.y * s;
                let y = state.velocity_momentum.x * s + state.velocity_momentum.y * c;
                state.velocity_momentum = Vec2 { x, y };
                let velocity_factor = clamp(elapsed_s * MAGIC_VELOCITY_CONSTANT, 0.0, 1.0);
                state.velocity_momentum =
                    state.velocity_momentum * (1.0 - velocity_factor) + pos_norm * velocity_factor;

                let mut rotate: f32;
                if ori_norm.y > 0.0 {
                    rotate = min(ori_norm.x, 1.0 - ori_norm.y);
                    if ori_norm.y > 0.5 {
                        rotate *= 0.9;
                    }
                } else if ori_norm.x > 0.0 {
                    rotate = 1.0;
                } else {
                    rotate = -1.0;
                }

                // Calculate a basic momentum term to compensate for overshooting
                rotate -= state.rotate_momentum;

                const MAGIC_ROTATION_CONSTANT: f32 = 5.0; // Magic constant describing how quickly momentum accumulates
                let rotate_factor = clamp(elapsed_s * MAGIC_ROTATION_CONSTANT, 0.0, 1.0);
                state.rotate_momentum =
                    state.rotate_momentum * (1.0 - rotate_factor) + rotate * rotate_factor;

                state.last_change = std::time::Instant::now();

                let mut max_abs_speed: f32 = 0.1;

                for wheel in &mut state.wheels {
                    let movement: f32 = clamp(wheel.wheel.forward.dot(&pos_norm), -1.0, 1.0);

                    let wheel_speed = movement + rotate;

                    wheel.speed = wheel_speed;

                    if abs(wheel_speed) > max_abs_speed {
                        max_abs_speed = abs(wheel_speed);
                    }
                }

                for wheel in &state.wheels {
                    wheel.tx.send(wheel.speed / max_abs_speed).unwrap();
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
            speed: 0.0,
            wheel: WheelExtrinsics {
                pivot: wheel.pivot,
                forward: wheel.forward,
            },
            tx: wheelcontrollers[index].wheelspeed_tx.clone(),
            extrinsics_rx: wheelcontrollers[index].extrinsics_rx.resubscribe(),
        });
    }

    MotionControllerNode {
        drop_rx: drop_rx.resubscribe(),
        movecommand_tx: tx,
        state: Arc::new(Mutex::new(MotionControllerState {
            last_change: std::time::Instant::now(),
            rotate_momentum: 0.0,
            velocity_momentum: Vec2 { x: 0.0, y: 0.0 },
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
