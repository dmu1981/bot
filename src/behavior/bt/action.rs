use crate::behavior::bt::*;

type ActionCallback<'a, T> = fn(&'a mut Box<T>) -> BTResult;

struct BTAction<'a, T> {
    callback: ActionCallback<'a, T>,
    decorators: Vec<BoxedDecorator<T>>,
}

impl<'a, T> BTAction<'a, T> {
    fn new(callback: ActionCallback<'a, T>) -> BTAction<'a, T> {
        BTAction {
            decorators: Vec::<BoxedDecorator<T>>::new(),
            callback,
        }
    }
}

impl<'a, T> BTNode<'a, T> for BTAction<'a, T> {
    fn reset(&'a mut self) {}

    fn get_decorators(&self) -> Iter<BoxedDecorator<T>> {
        self.decorators.iter()
    }

    fn internal_tick(&'a mut self, blackboard: &'a mut Box<T>) -> BTResult {
        (self.callback)(blackboard)
    }
}
