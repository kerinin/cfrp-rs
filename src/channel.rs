use super::{Channel, InternalSignal, Push, Event};

impl<A> InternalSignal<A> for Channel<A> where
    A: 'static + Send,
{
    fn push_to(self: Box<Self>, mut target: Box<Push<A>>) {
        loop {
            match self.source_rx.recv() {
                Err(_) => {
                    target.push(Event::Exit);

                    return
                },
                Ok(a) => target.push(a),
            }
        }
    }
}
