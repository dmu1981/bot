use crate::math::Vec2;
use crate::motioncontroll::MoveCommand;
use crate::node::*;
use crate::perception::PerceptionMessage;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

struct BehaviorState {
    movecommand_tx: Sender<MoveCommand>,
}

pub struct BehaviorNode {
    drop_rx: Receiver<()>,
    state: Arc<Mutex<BehaviorState>>,
    perception_rx: Receiver<PerceptionMessage>,
}

fn on_perception(perception: PerceptionMessage, state: State<BehaviorState>) -> DynFut<NodeResult> {
    Box::pin(async move {
        if perception.ball.position.is_none() || perception.target_goal.position.is_none() {
            panic!("Cannot handle missing measurements yet.");
        }

        //let ball = perception.ball.position.or_else(|| -> Vec2 { x: 0.0, y: 0.0 });
        //let goal = perception.target_goal.position.or_else(|| -> Vec2 { x: 0.0, y: 0.0 });

        if let Some(ball_position) = perception.ball.position {
            state
                .movecommand_tx
                .send(MoveCommand::MoveAndAlign(
                    ball_position,
                    Vec2 { x: 0.0, y: 1.0 },
                ))
                .unwrap();
        }

        Ok(ThreadNext::Next)
    })
}

pub fn create(
    perception_rx: Receiver<PerceptionMessage>,
    movecommand_tx: Sender<MoveCommand>,
    drop_tx: Sender<()>,
) -> BehaviorNode {
    BehaviorNode {
        drop_rx: drop_tx.subscribe(),
        perception_rx,
        state: Arc::new(Mutex::new(BehaviorState { movecommand_tx })),
    }
}

impl Node<BehaviorState> for BehaviorNode {
    fn get_state_handle(&self) -> Arc<Mutex<BehaviorState>> {
        self.state.clone()
    }

    fn get_drop_rx(&self) -> Receiver<()> {
        self.drop_rx.resubscribe()
    }
}

#[async_trait]
impl Executor for BehaviorNode {
    async fn init(&self) -> Handles {
        vec![]
    }

    async fn run(&self) -> Handles {
        vec![self.subscribe(self.perception_rx.resubscribe(), on_perception)]
    }

    async fn stop(&self) -> Handles {
        vec![]
    }
}
