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

pub type BoxedNode<T> = Box<dyn BTNode<T>>;
pub type BoxedDecorator<T> = Box<dyn BTDecorator<T>>;

pub struct BehaviorTree<T> {
    blackboard: Box<T>,
    root: BoxedNode<T>,
}

unsafe impl<T> Send for BehaviorTree<T> where T: Send {}
unsafe impl<T> Sync for BehaviorTree<T> where T: Sync {}

impl<'a, T> BehaviorTree<T> {
    pub fn new(root: BoxedNode<T>, bb: Box<T>) -> BehaviorTree<T> {
        BehaviorTree {
            blackboard: bb,
            root,
        }
    }

    pub fn get_blackboard(&mut self) -> &mut T {
        self.blackboard.as_mut()
    }

    fn reset(&'a mut self) {
        self.root.reset();
    }

    pub fn tick(&'a mut self) -> BTResult {
        self.root.tick(&mut self.blackboard)
    }
}

pub trait BTNode<T> {
    fn reset(&mut self) {}

    fn tick(&mut self, blackboard: &mut Box<T>) -> BTResult {
        if let BTDecoratorResult::Failure = self.check_decorators(blackboard) {
            return BTResult::Failure;
        };

        self.internal_tick(blackboard)
    }

    fn internal_tick(&mut self, blackboard: &mut Box<T>) -> BTResult;

    fn get_decorators(&self) -> Iter<Box<dyn BTDecorator<T>>>;

    fn check_decorators(&self, blackboard: &T) -> BTDecoratorResult {
        for decorator in self.get_decorators() {
            if let BTDecoratorResult::Failure = decorator.evaluate(blackboard) {
                return BTDecoratorResult::Failure;
            }
        }

        BTDecoratorResult::Success
    }
}

pub trait BTDecorator<T> {
    fn evaluate(&self, blackboard: &T) -> BTDecoratorResult;
}

pub mod action;
pub mod sequence;

pub use action::*;
