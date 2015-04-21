use std::marker::*;

use super::{Fold, Signal, Push, Event};

impl<F, A, B> Signal<B> for Fold<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    fn push_to(self: Box<Self>, target: Box<Push<B>>) {
        let inner = *self;
        let Fold {parent, f, state} = inner;

        parent.push_to(
            Box::new(
                FoldPusher {
                    child: target,
                    f: f,
                    state: state,
                    marker: PhantomData,
                }
            )
        );
    }
}

pub struct FoldPusher<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    child: Box<Push<B>>,
    f: F,
    state: B,
    marker: PhantomData<A>,
}

impl<F, A, B> Push<A> for FoldPusher<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    fn push(&mut self, event: Event<A>) {
        let out = match event {
            Event::Changed(a) => { 
                (self.f)(&mut self.state, a);
                Event::Changed(self.state.clone())
            },
            Event::Unchanged => Event::Unchanged,
            Event::NoOp => Event::NoOp,
            Event::Exit => Event::Exit,
        };
        self.child.push(out);
    }
}
