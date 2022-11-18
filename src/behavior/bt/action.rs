use crate::behavior::bt::*;

type ActionCallback<T> = fn(&mut Box<T>) -> BTResult;

pub struct BTAction<T> {
    callback: ActionCallback<T>,
    decorators: Vec<BoxedDecorator<T>>,
}

impl<T> BTAction<T> {
    pub fn new(callback: ActionCallback<T>) -> BTAction<T> {
        BTAction {
            decorators: Vec::<BoxedDecorator<T>>::new(),
            callback,
        }
    }
}

impl<T> BTNode<T> for BTAction<T> {
    fn reset(&mut self) {}

    fn get_decorators(&self) -> Iter<BoxedDecorator<T>> {
        self.decorators.iter()
    }

    fn internal_tick(&mut self, blackboard: &mut Box<T>) -> BTResult {
        (self.callback)(blackboard)
    }
}
