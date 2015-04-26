use std::marker::*;

use super::super::{Event, Signal, SignalType, Push, Lift, Lift2, Fold};

/// The result of a `lift` operation
///
pub struct LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    parent: Box<Signal<A>>,
    f: F,
    initial: B,
}

impl<F, A, B> LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    pub fn new(parent: Box<Signal<A>>, f: F, initial: B) -> Self {
        LiftSignal {
            parent: parent, 
            f: f,
            initial: initial,
        }
    }
}

impl<F, A, B> Signal<B> for LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    fn initial(&self) -> SignalType<B> {
        SignalType::Dynamic(self.initial.clone())
    }

    fn push_to(self: Box<Self>, target: Option<Box<Push<B>>>) {
        let inner = *self;
        let LiftSignal { parent, f, initial } = inner;

        match target {
            Some(t) => {
                debug!("Lift::push_to Some");

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
            None => {
                debug!("Lift::push_to None");

                parent.push_to(None)
            },
        }
    }
}

impl<F, A, B> Lift<B> for LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send + Clone,
{}
impl<F, A, B, C, SC> Lift2<B, C, SC> for LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send + Clone,
{}
impl<F, A, B> Fold<B> for LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send + Clone,
{}


struct LiftPusher<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    child: Box<Push<B>>,
    f: F,
    marker: PhantomData<A>,
}

impl<F, A, B> Push<A> for LiftPusher<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    fn push(&mut self, event: Event<A>) {
        let out = match event {
            Event::Changed(a) => {
                debug!("LiftPusher handling Event::Changed");
                let b = (self.f)(a);
                Event::Changed(b)
            },
            Event::Unchanged => {
                debug!("LiftPusher handling Event::Unchanged");
                Event::Unchanged
            },
            Event::Exit => {
                debug!("LiftPusher handling Event::Exit");
                Event::Exit
            },
        };

        self.child.push(out);
    }
}
