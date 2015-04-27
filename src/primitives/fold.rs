use std::marker::*;

use super::super::{Event, Signal, SignalType, Push};

/// The result of a `fold` operation
///
pub struct FoldSignal<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    parent: Box<Signal<A>>,
    f: F,
    state: B,
}

impl<F, A, B> FoldSignal<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    pub fn new(parent: Box<Signal<A>>, initial: B, f: F) -> Self {
        FoldSignal {
            parent: parent, 
            f: f,
            state: initial,
        }
    }
}

impl<F, A, B> Signal<B> for FoldSignal<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
{
    fn initial(&self) -> SignalType<B> {
        SignalType::Dynamic(self.state.clone())
    }

    fn push_to(self: Box<Self>, target: Option<Box<Push<B>>>) {
        let inner = *self;
        let FoldSignal {parent, f, state} = inner;

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

/*
#[cfg(test)] 
mod test {
    extern crate env_logger;

    use std::sync::mpsc::*;

    use super::super::super::{Signal, SignalExt};
    use super::super::channel::Channel;
    use super::super::value::Value;
    use super::super::fold::FoldSignal;

    // env_logger::init().unwrap();

    #[test]
    fn fold_constructs_from_value() {
        let v = Value::new(0);
        let f = FoldSignal::new(Box::new(v), 0, |s, i| { *s += i });

        assert!(true);
    }

    #[test]
    fn fold_constructs_from_channel() {
        let (tx, rx) = channel();
        let c = Channel::new(rx, 0);
        let f = FoldSignal::new(Box::new(c), 0, |s, i| { *s += i });

        assert!(true);
    }

    #[test]
    fn fold_impls_fold() {
        let v = Value::new(0);
        let f = Box::new(FoldSignal::new(Box::new(v), 0, |s, i| { *s += i }));
        let f2 = f.fold(0, |s, i| { *s += i });

        assert!(true);
    }

    #[test]
    fn fold_impls_lift() {
        let v = Value::new(0);
        let f = Box::new(FoldSignal::new(Box::new(v), 0, |s, i| { *s += i }));
        let f2 = f.lift(|i| { i + 1 });

        assert!(true);
    }
}
*/
