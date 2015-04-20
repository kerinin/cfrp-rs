use std::cell::*;

use super::{Signal, Run};

pub struct Fold<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    parent: Box<Signal<A> + Send>,
    f: RefCell<F>,
    state: RefCell<B>,
}

impl<F, A, B> Fold<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    pub fn new(parent: Box<Signal<A> + Send>, f: F, initial: B) -> Fold<F, A, B> {
        Fold {
            parent: parent,
            f: RefCell::new(f),
            state: RefCell::new(initial),
        }
    }
}

impl<F, A, B> Signal<B> for Fold<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    fn recv(&self) -> Option<B> {
       match self.parent.recv() {
           Some(a) => { 
               let mut f: &mut F = &mut self.f.borrow_mut();
               f(&mut self.state.borrow_mut(), a);
               Some(self.state.borrow().clone())
            },
           None => None,
       }
    }
}

impl<F, A, B> Run for Fold<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    fn run(self: Box<Self>) {
        loop {
            self.recv();
        }
    }
}
