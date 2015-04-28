use std::marker::*;

use super::super::{Event, Signal, SignalExt, SignalType, Push, Config};

/// The result of a `fold` operation
///
pub struct FoldSignal<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
{
    config: Config,
    parent: Box<Signal<A>>,
    f: F,
    state: SignalType<B>,
}

impl<F, A, B> FoldSignal<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
{
    pub fn new(config: Config, parent: Box<Signal<A>>, mut initial: B, mut f: F) -> Self {
        let state = match parent.initial() {
            SignalType::Constant(a) => {
                f(&mut initial, a);
                SignalType::Constant(initial)
            },
            SignalType::Dynamic(a) => {
                f(&mut initial, a);
                SignalType::Dynamic(initial)
            },
        };

        FoldSignal {
            config: config,
            parent: parent, 
            f: f,
            state: state,
        }
    }
}

impl<F, A, B> Signal<B> for FoldSignal<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
{
    fn config(&self) -> Config {
        self.config.clone()
    }

    fn initial(&self) -> SignalType<B> {
        self.state.clone()
    }

    fn push_to(self: Box<Self>, target: Option<Box<Push<B>>>) {
        let inner = *self;
        let FoldSignal {config: _, parent, f, state} = inner;

        let s = match state {
            SignalType::Constant(s) => s,
            SignalType::Dynamic(s) => s,
        };

        match target {
            Some(t) => {
                parent.push_to(
                    Some(
                        Box::new(
                            FoldPusher {
                                child: Some(t),
                                f: f,
                                state: s,
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
                                state: s,
                                marker: PhantomData,
                            }
                        )
                    )
                );
            }
        }
    }
}
impl<F, A, B> SignalExt<B> for FoldSignal<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
{}

struct FoldPusher<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
{
    child: Option<Box<Push<B>>>,
    f: F,
    state: B,
    marker: PhantomData<A>,
}

impl<F, A, B> Push<A> for FoldPusher<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
{
    fn push(&mut self, event: Event<A>) {
        let out = match event {
            Event::Changed(a) => { 
                debug!("FoldPusher handling Event::Changed");
                (self.f)(&mut self.state, a);
                Event::Changed(self.state.clone())
            },
            Event::Unchanged => {
                debug!("FoldPusher handling Event::Unchanged");
                Event::Unchanged
            },
            Event::Exit => {
                debug!("FoldPusher handling Event::Exit");
                Event::Exit
            },
        };

        match self.child {
            Some(ref mut c) => c.push(out),
            None => {},
        }
    }
}
