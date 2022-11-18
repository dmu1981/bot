use crate::behavior::bt::*;

struct BTSequence<T> {
    nodes: Vec<BoxedNode<T>>,
    index: usize,
    decorators: Vec<BoxedDecorator<T>>,
}

impl<T> BTSequence<T> {
    fn new(nodes: Vec<BoxedNode<T>>) -> BTSequence<T> {
        BTSequence {
            decorators: Vec::<BoxedDecorator<T>>::new(),
            nodes,
            index: 0,
        }
    }
}

impl<T> BTNode<T> for BTSequence<T> {
    fn reset(&mut self) {
        self.index = 0;
    }

    fn get_decorators(&self) -> Iter<Box<dyn BTDecorator<T>>> {
        self.decorators.iter()
    }

    fn internal_tick(&mut self, blackboard: &mut Box<T>) -> BTResult {
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
