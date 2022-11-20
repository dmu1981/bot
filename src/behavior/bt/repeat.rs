use crate::behavior::bt::*;

pub struct BTRepeat<T> {
    node: BoxedNode<T>,
    decorators: Vec<BoxedDecorator<T>>,
    counter: Option<u32>,
}

impl<T> BTRepeat<T> {
    pub fn new(counter: Option<u32>, node: BoxedNode<T>) -> Box<BTRepeat<T>> {
        Box::new(BTRepeat {
            decorators: Vec::<BoxedDecorator<T>>::new(),
            node,
            counter,
        })
    }
}

impl<T> BTNode<T> for BTRepeat<T> {
    fn reset(&mut self) {
        self.node.reset();
    }

    fn get_decorators(&self) -> Iter<BoxedDecorator<T>> {
        self.decorators.iter()
    }

    fn internal_tick(&mut self, blackboard: &mut Box<T>) -> BTResult {
        let result = self.node.tick(blackboard);

        match result {
            BTResult::Failure => match self.counter {
                Some(value) => {
                    if value > 0 {
                        self.counter = Some(value - 1);
                        self.reset();
                        BTResult::Pending
                    } else {
                        BTResult::Failure
                    }
                }
                None => {
                    self.reset();
                    BTResult::Pending
                }
            },
            BTResult::Success => BTResult::Success,
            BTResult::Pending => BTResult::Pending,
        }
    }
}
