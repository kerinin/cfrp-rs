use std::marker::*;

use super::{Lift, InternalSignal, Push, Event};

impl<F, A, B> InternalSignal<B> for Lift<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{
    fn push_to(self: Box<Self>, target: Option<Box<Push<B>>>) {
        let inner = *self;
        let Lift { parent, f } = inner;

        match target {
            Some(t) => {
                parent.push_to(
                    Some(
                        Box::new(
                            LiftPusher {
                                child: t,
                                f: f,
                                marker: PhantomData,
                            }
                        )
                    )
                );
            },
            None => parent.push_to(None),
        }
    }
}

struct LiftPusher<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{
    child: Box<Push<B>>,
    f: F,
    marker: PhantomData<A>,
}

impl<F, A, B> Push<A> for LiftPusher<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{
    fn push(&mut self, event: Event<A>) {
        let out = match event {
            Event::Changed(a) => {
                let b = (self.f)(a);
                Event::Changed(b)
            },
            Event::Unchanged => Event::Unchanged,
            Event::NoOp => Event::NoOp,
            Event::Exit => Event::Exit,
        };
        self.child.push(out);
    }
}
