use super::{Channel, InternalSignal, Push, Event};

impl<A> InternalSignal<A> for Channel<A> where
    A: 'static + Send,
{
    fn push_to(self: Box<Self>, target: Option<Box<Push<A>>>) {
        match target {
            Some(mut t) => {
                loop {
                    match self.source_rx.recv() {
                        Err(_) => {
                            t.push(Event::Exit);

                            return
                        },
                        Ok(a) => t.push(a),
                    }
                }
            }
            None => {
                // Just ensuring the channel is drained so we don't get memory leaks
                loop {
                    match self.source_rx.recv() {
                        Err(_) => return,
                        _ => {},
                    }
                }
            },
        }
    }
}
