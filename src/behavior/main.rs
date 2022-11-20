use crate::behavior::bt::*;
use crate::behavior::move_into_shoot_position::*;
use crate::behavior::shoot_into_goal::*;
use crate::math::Vec2;
use crate::motioncontroll::MoveCommand;
use crate::node::*;
use crate::perception::PerceptionMessage;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

pub struct MyBlackboard {
    pub ball: Vec2,
    pub target_goal: Vec2,
    pub movecommand_tx: Sender<MoveCommand>,
    pub kicker_tx: Sender<bool>,
}

struct BehaviorState {
    tree: BehaviorTree<MyBlackboard>,
}

pub struct BehaviorNode {
    drop_rx: Receiver<()>,
    state: Arc<Mutex<BehaviorState>>,
    perception_rx: Receiver<PerceptionMessage>,
}

fn on_perception(
    perception: PerceptionMessage,
    mut state: State<BehaviorState>,
) -> DynFut<NodeResult> {
    Box::pin(async move {
        if perception.ball.position.is_none() || perception.target_goal.position.is_none() {
            panic!("Cannot handle missing measurements yet.");
        }

        // Copy measurements into the behavior tree blackboard
        state.tree.get_blackboard().ball = perception.ball.position.unwrap();
        state.tree.get_blackboard().target_goal = perception.target_goal.position.unwrap();

        state.tree.tick();

        /*state
        .movecommand_tx
        .send(MoveCommand::MoveAndAlign(ball, goal))
        .unwrap();*/

        Ok(ThreadNext::Next)
    })
}

fn on_interval(mut state: State<BehaviorState>) -> DynFut<NodeResult> {
    Box::pin(async move {
        state.tree.tick();

        Ok(ThreadNext::Next)
    })
}

pub fn create(
    perception_rx: Receiver<PerceptionMessage>,
    movecommand_tx: Sender<MoveCommand>,
    kicker_tx: Sender<bool>,
    drop_tx: Sender<()>,
) -> BehaviorNode {
    let bb = MyBlackboard {
        movecommand_tx,
        kicker_tx,
        ball: Vec2 { x: 0.0, y: 0.0 },
        target_goal: Vec2 { x: 0.0, y: 0.0 },
    };

    let root = BTRepeat::new(
        None,
        BTSequence::new(vec![BTMoveIntoShootPosition::new(), BTShootIntoGoal::new()]),
    );

    let tree = BehaviorTree::new(root, Box::new(bb));

    BehaviorNode {
        drop_rx: drop_tx.subscribe(),
        perception_rx,
        state: Arc::new(Mutex::new(BehaviorState { tree })),
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
        vec![
            self.subscribe(self.perception_rx.resubscribe(), on_perception),
            //self.interval(Duration::from_millis(60), on_interval)
        ]
    }

    async fn stop(&self) -> Handles {
        vec![]
    }
}
