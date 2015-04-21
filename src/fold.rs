use std::cell::*;

use super::{Fold, Signal, Run};

impl<F, A, B> Signal<B> for Fold<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    fn recv(&mut self) -> Option<B> {
        let received = self.parent.recv();

        match received {
            Some(a) => { 
                (self.f)(&mut self.state);
                Some(self.state.clone())
            },
            None => {
                None
            },
        }
    }
}

impl<F, A, B> Run for Fold<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    fn run(mut self: Box<Self>) {
        loop {
            self.recv();
        }
    }
}
