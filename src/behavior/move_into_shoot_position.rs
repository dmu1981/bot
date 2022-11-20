use crate::behavior::bt::*;
use crate::behavior::MyBlackboard;
use crate::math::Vec2;
use crate::motioncontroll::MoveCommand;
use std::slice::Iter;

pub struct BTMoveIntoShootPosition {
    is_between_ball_and_goal: bool,
    decorators: Vec<BoxedDecorator<MyBlackboard>>,
}

impl BTMoveIntoShootPosition {
    pub fn new() -> Box<BTMoveIntoShootPosition> {
        Box::new(BTMoveIntoShootPosition {
            is_between_ball_and_goal: true,
            decorators: Vec::<BoxedDecorator<MyBlackboard>>::new(),
        })
    }
}

impl BTNode<MyBlackboard> for BTMoveIntoShootPosition {
    fn get_decorators(&self) -> Iter<Box<dyn BTDecorator<MyBlackboard>>> {
        self.decorators.iter()
    }

    fn internal_tick(&mut self, blackboard: &mut Box<MyBlackboard>) -> BTResult {
        const DISTANCE_THRESHOLD: f32 = 0.5;
        const DOT_PRODUCT_THRESHOLD: f32 = 0.98;
        const SHOT_RANGE: f32 = 2.0;

        let ball_norm = blackboard.ball.normalize();
        let goal_norm = blackboard.target_goal.normalize();
        let goal_to_ball = (blackboard.ball - blackboard.target_goal).normalize();

        let dot = ball_norm.dot(&goal_norm);

        // Small hysteresis
        if self.is_between_ball_and_goal && dot > 0.3 {
            self.is_between_ball_and_goal = false;
        } else if !self.is_between_ball_and_goal && dot < -0.3 {
            self.is_between_ball_and_goal = true;
        }

        //println!("Between ball and goal: {}", self.is_between_ball_and_goal);

        let target_position: Vec2 = if self.is_between_ball_and_goal {
            let mut normal = Vec2 {
                x: goal_to_ball.y,
                y: -goal_to_ball.x,
            };

            if normal.dot(&ball_norm) > 0.0 {
                normal *= -1.0;
            }

            blackboard.ball + (goal_to_ball + normal) * SHOT_RANGE
        } else {
            blackboard.ball + goal_to_ball * SHOT_RANGE
        };

        let target_orientation = -goal_to_ball;

        blackboard
            .movecommand_tx
            .send(MoveCommand::MoveAndAlign(
                target_position,
                target_orientation,
            ))
            .unwrap();

        if target_position.magnitude() < DISTANCE_THRESHOLD
            && target_orientation.y > DOT_PRODUCT_THRESHOLD
        {
            BTResult::Success
        } else {
            BTResult::Pending
        }
    }
}
