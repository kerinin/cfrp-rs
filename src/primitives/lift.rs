use std::marker::*;

use super::super::{Event, Signal, SignalExt, SignalType, Push, Config};

/// The result of a `lift` operation
///
pub struct LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
{
    config: Config,
    parent: Box<Signal<A>>,
    f: F,
    initial: SignalType<B>,
}

impl<F, A, B> LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
{
    pub fn new(config: Config, parent: Box<Signal<A>>, f: F) -> Self {
        let initial = match parent.initial() {
            SignalType::Constant(a) => SignalType::Constant(f(a)),
            SignalType::Dynamic(a) => SignalType::Dynamic(f(a)),
        };

        LiftSignal {
            config: config,
            parent: parent, 
            f: f,
            initial: initial,
        }
    }
}

impl<F, A, B> Signal<B> for LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
{
    fn config(&self) -> Config {
        self.config.clone()
    }

    fn initial(&self) -> SignalType<B> {
        self.initial.clone()
    }

    fn push_to(self: Box<Self>, target: Option<Box<Push<B>>>) {
        let inner = *self;
        let LiftSignal { config: _, parent, f, initial: _ } = inner;

        match target {
            Some(t) => {
                debug!("SETUP: Sending to target Some");
                parent.push_to(
                    Some(
                        Box::new(
                            LiftPusher {
                                child: Some(t),
                                f: f,
                                marker: PhantomData,
                            }
                        )
                    )
                );
            }

            None => {
                debug!("SETUP: Sending to target None");
                parent.push_to(
                    Some(
                        Box::new(
                            LiftPusher {
                                child: None,
                                f: f,
                                marker: PhantomData,
                            }
                        )
                    )
                );
            }
        }
    }
}
impl<F, A, B> SignalExt<B> for LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
{}

struct LiftPusher<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
{
    child: Option<Box<Push<B>>>,
    f: F,
    marker: PhantomData<A>,
}

impl<F, A, B> Push<A> for LiftPusher<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
{
    fn push(&mut self, event: Event<A>) {
        let out = match event {
            Event::Changed(a) => {
                info!("RUN: LiftPusher handling Event::Changed");
                let b = (self.f)(a);
                Event::Changed(b)
            },
            Event::Unchanged => {
                info!("RUN: LiftPusher handling Event::Unchanged");
                Event::Unchanged
            },
            Event::Exit => {
                info!("RUN: LiftPusher handling Event::Exit");
                Event::Exit
            },
        };

        match self.child {
            Some(ref mut t) => t.push(out),
            None => {},
        }
    }
}
