use crate::behavior::bt::*;

struct BTSequence<'a, T> {
    nodes: Vec<BoxedNode<'a, T>>,
    index: usize,
    decorators: Vec<BoxedDecorator<T>>,
}

impl<'a, T> BTSequence<'a, T> {
    fn new(nodes: Vec<BoxedNode<'a, T>>) -> BTSequence<'a, T> {
        BTSequence {
            decorators: Vec::<BoxedDecorator<T>>::new(),
            nodes,
            index: 0,
        }
    }
}

impl<'a, T> BTNode<'a, T> for BTSequence<'a, T> {
    fn reset(&'a mut self) {
        self.index = 0;
    }

    fn get_decorators(&self) -> Iter<Box<dyn BTDecorator<T>>> {
        self.decorators.iter()
    }

    fn internal_tick(&'a mut self, blackboard: &'a mut Box<T>) -> BTResult {
        let len = self.nodes.len();

        if self.index < len {
            match self.nodes[self.index].as_mut().tick(blackboard) {
                BTResult::Success => {
                    self.index += 1;
                    if self.index < len {
                        return BTResult::Pending;
                    } else {
                        return BTResult::Success;
                    }
                }
                BTResult::Pending => {
                    return BTResult::Pending;
                }
                BTResult::Failure => {
                    return BTResult::Failure;
                }
            }
        }
        {
            BTResult::Failure
        }
    }
}
