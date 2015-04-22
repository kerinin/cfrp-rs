use std::marker::*;

use super::{Fold, InternalSignal, Push, Event};

impl<F, A, B> InternalSignal<B> for Fold<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    fn push_to(self: Box<Self>, target: Option<Box<Push<B>>>) {
        let inner = *self;
        let Fold {parent, f, state} = inner;

        match target {
            Some(t) => {
                parent.push_to(
                    Some(
                        Box::new(
                            FoldPusher {
                                child: Some(t),
                                f: f,
                                state: state,
                                marker: PhantomData,
                            }
                        )
                    )
                );
            },
            None => {
                parent.push_to(
                    Some(
                        Box::new(
                            FoldPusher {
                                child: None,
                                f: f,
                                state: state,
                                marker: PhantomData,
                            }
                        )
                    )
                );
            }
        }
    }
}

struct FoldPusher<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    child: Option<Box<Push<B>>>,
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
        match self.child {
            Some(ref mut c) => c.push(out),
            None => {},
        }
    }
}
