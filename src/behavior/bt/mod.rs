use std::slice::Iter;

pub enum BTResult {
    Success,
    Failure,
    Pending,
}

pub enum BTDecoratorResult {
    Success,
    Failure,
}

type BoxedNode<'a, T> = Box<dyn BTNode<'a, T>>;
type BoxedDecorator<T> = Box<dyn BTDecorator<T>>;

pub struct BehaviorTree<'a, T> {
    blackboard: Box<T>,
    root: BoxedNode<'a, T>,
}

impl<'a, T> BehaviorTree<'a, T> {
    fn new(root: BoxedNode<'a, T>, bb: Box<T>) -> BehaviorTree<'a, T> {
        BehaviorTree {
            blackboard: bb,
            root,
        }
    }

    fn reset(&'a mut self) {
        self.root.reset();
    }

    fn tick(&'a mut self) -> BTResult {
        self.root.tick(&mut self.blackboard)
    }
}

pub trait BTNode<'a, T> {
    fn reset(&'a mut self);

    fn tick(&'a mut self, blackboard: &'a mut Box<T>) -> BTResult {
        match self.check_decorators(blackboard) {
            BTDecoratorResult::Failure => {
                return BTResult::Failure;
            }
            _ => {}
        };

        self.internal_tick(blackboard)
    }

    fn internal_tick(&'a mut self, blackboard: &'a mut Box<T>) -> BTResult;

    fn get_decorators(&self) -> Iter<Box<dyn BTDecorator<T>>>;

    fn check_decorators(&self, blackboard: &Box<T>) -> BTDecoratorResult {
        for decorator in self.get_decorators() {
            match decorator.evaluate(blackboard) {
                BTDecoratorResult::Failure => {
                    return BTDecoratorResult::Failure;
                }
                _ => {}
            }
        }

        return BTDecoratorResult::Success;
    }
}

pub trait BTDecorator<T> {
    fn evaluate(&self, blackboard: &Box<T>) -> BTDecoratorResult;
}

pub mod action;
pub mod sequence;
