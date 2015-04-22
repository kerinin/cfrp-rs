use std::marker::*;

use super::{Lift, InternalSignal, Push, Event};

impl<F, A, B> InternalSignal<B> for Lift<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{
    fn push_to(self: Box<Self>, target: Box<Push<B>>) {
        let inner = *self;
        let Lift { parent, f } = inner;

        parent.push_to(
            Box::new(
                LiftPusher {
                    child: target,
                    f: f,
                    marker: PhantomData,
                }
            )
        );
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
