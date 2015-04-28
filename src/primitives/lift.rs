use std::marker::*;

use super::super::{Event, Signal, SignalExt, SignalType, Push};

/// The result of a `lift` operation
///
pub struct LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
{
    parent: Box<Signal<A>>,
    f: F,
    initial: SignalType<B>,
}

impl<F, A, B> LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
{
    pub fn new(parent: Box<Signal<A>>, f: F) -> Self {
        let initial = match parent.initial() {
            SignalType::Constant(a) => SignalType::Constant(f(a)),
            SignalType::Dynamic(a) => SignalType::Dynamic(f(a)),
        };

        LiftSignal {
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
    fn initial(&self) -> SignalType<B> {
        self.initial.clone()
    }

    fn push_to(self: Box<Self>, target: Option<Box<Push<B>>>) {
        let inner = *self;
        let LiftSignal { parent, f, initial: _ } = inner;

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

    /*
#[cfg(test)] 
mod test {
    extern crate env_logger;

    use std::sync::mpsc::*;

    use super::super::super::{Signal, SignalExt};
    use super::super::channel::Channel;
    use super::super::value::Value;
    use super::super::lift::LiftSignal;

    // env_logger::init().unwrap();

    #[test]
    fn lift_constructs_from_value() {
        let v = Value::new(0);
        let l = LiftSignal::new(Box::new(v), |i| -> usize { i + 1 }, 0);

        assert!(true);
    }

    #[test]
    fn lift_constructs_from_channel() {
        let (tx, rx) = channel();
        let c = Channel::new(rx, 0);
        let l = LiftSignal::new(Box::new(c), |i| { i + 1 }, 0);

        assert!(true);
    }

    #[test]
    fn lift_lifts_from_value() {
        let v = Value::new(0);
        let l = Box::new(LiftSignal::new(Box::new(v), |i| { i + 1 }, 0));
        let l2 = l.lift(|i| { i + 1 });

        assert!(true);
    }

    #[test]
    fn lift_lifts_from_channel() {
        let (tx, rx) = channel();
        let c: Box<Signal<usize>> = Box::new(Channel::new(rx, 0));
        let l: Box<Signal<usize>> = Box::new(LiftSignal::new(c, |i| { i + 1 }, 0));
        let l2 = l.lift(|i| { i + 1 });

        assert!(true);
    }
}
    */
