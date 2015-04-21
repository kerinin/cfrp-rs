use super::{Lift, Signal, Run, Event};

impl<F, A, B> Signal<B> for Lift<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{
    fn recv(&mut self) -> Event<B> {
        let received = self.parent.recv();
        match received {
            Event::Changed(a) => {
                let b = (self.f)(a);
                Event::Changed(b)
            },
            Event::Unchanged => Event::Unchanged,
            Event::NoOp => Event::NoOp,
            Event::Exit => Event::Exit,
        }
    }
}
