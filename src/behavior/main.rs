use crate::behavior::bt::*;
use crate::math::Vec2;
use crate::motioncontroll::MoveCommand;
use crate::node::*;
use crate::perception::PerceptionMessage;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

struct MyBlackboard {
    ball: Vec2,
    target_goal: Vec2,
}

struct BehaviorState {
    movecommand_tx: Sender<MoveCommand>,
    tree: BehaviorTree<MyBlackboard>,
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

        let ball = perception.ball.position.unwrap();
        let goal = perception.target_goal.position.unwrap();

        state
            .movecommand_tx
            .send(MoveCommand::MoveAndAlign(ball, goal))
            .unwrap();

        Ok(ThreadNext::Next)
    })
}

fn cb(_bb: &mut Box<MyBlackboard>) -> BTResult {
    BTResult::Pending
}

pub fn create(
    perception_rx: Receiver<PerceptionMessage>,
    movecommand_tx: Sender<MoveCommand>,
    drop_tx: Sender<()>,
) -> BehaviorNode {
    let bb = MyBlackboard {
        ball: Vec2 { x: 0.0, y: 0.0 },
        target_goal: Vec2 { x: 0.0, y: 0.0 },
    };
    let root = BTAction::<MyBlackboard>::new(cb);

    let tree = BehaviorTree::new(Box::new(root), Box::new(bb));
    BehaviorNode {
        drop_rx: drop_tx.subscribe(),
        perception_rx,
        state: Arc::new(Mutex::new(BehaviorState {
            movecommand_tx,
            tree,
        })),
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
