use crate::behavior::bt::*;
use crate::behavior::MyBlackboard;
use crate::math::abs;
use crate::motioncontroll::MoveCommand;
use std::slice::Iter;

pub struct BTShootIntoGoal {
    decorators: Vec<BoxedDecorator<MyBlackboard>>,
}

impl BTShootIntoGoal {
    pub fn new() -> Box<BTShootIntoGoal> {
        Box::new(BTShootIntoGoal {
            decorators: Vec::<BoxedDecorator<MyBlackboard>>::new(),
        })
    }
}

impl BTNode<MyBlackboard> for BTShootIntoGoal {
    fn get_decorators(&self) -> Iter<Box<dyn BTDecorator<MyBlackboard>>> {
        self.decorators.iter()
    }

    fn internal_tick(&mut self, blackboard: &mut Box<MyBlackboard>) -> BTResult {
        const DISTANCE_THRESHOLD: f32 = 0.5;
        const DOT_PRODUCT_THRESHOLD: f32 = 0.7;
        const SHOT_RANGE: f32 = 2.0;

        let ball = blackboard.ball;
        let ball_norm = ball.normalize();

        let goal = blackboard.target_goal;

        let goal_norm = goal.normalize();

        let error = blackboard.ball - goal_norm * 1.1;
        let mut target_pos = blackboard.ball + error * 1.5;
        target_pos = target_pos.normalize() * 5.0;

        //let goal_norm = blackboard.target_goal.normalize();
        //let goal_to_ball = (blackboard.ball - blackboard.target_goal).normalize();

        let target_position = target_pos; //blackboard.target_goal;
        let target_orientation = goal;

        if abs(blackboard.ball.x) < 0.2
            && abs(blackboard.ball.y - 1.2) < 0.2
            && blackboard.target_goal.magnitude() < 15.0
        {
            blackboard.kicker_tx.send(true).unwrap();
        }

        blackboard
            .movecommand_tx
            .send(MoveCommand::MoveAndAlign(
                target_position,
                target_orientation,
            ))
            .unwrap();

        if ball_norm.dot(&target_orientation) < DOT_PRODUCT_THRESHOLD {
            BTResult::Failure
        } else if target_position.magnitude() < DISTANCE_THRESHOLD {
            BTResult::Success
        } else {
            BTResult::Pending
        }
    }
}
