use std::cell::*;

use super::{Fold, Signal, Run, Event};

impl<F, A, B> Signal<B> for Fold<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    fn recv(&mut self) -> Event<B> {
        let received = self.parent.recv();

        match received {
            Event::Changed(a) => { 
                (self.f)(&mut self.state, a);
                Event::Changed(self.state.clone())
            },
            Event::Unchanged => Event::Unchanged,
            Event::NoOp => Event::NoOp,
            Event::Exit => Event::Exit,
        }
    }
}
