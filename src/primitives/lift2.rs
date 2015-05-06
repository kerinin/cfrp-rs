use std::thread;
use std::sync::mpsc::*;

use super::super::{Event, Signal, SignalExt, SignalType, Push, Config};

pub enum Value<T> {
    Changed(T),
    Unchanged(T),
}

impl<T> Value<T> {
    pub fn unwrap(self) -> T {
        match self {
            Value::Changed(v) => v,
            Value::Unchanged(v) => v,
        }
    }
}

impl<T> Clone for Value<T> where T: Clone {
    fn clone(&self) -> Self {
        match self {
            &Value::Changed(ref v) => Value::Changed(v.clone()),
            &Value::Unchanged(ref v) => Value::Unchanged(v.clone()),
        }
    }
}

/// The result of a `lift2` operation
///
pub struct Lift2Signal<F, A, B, C> where
    F: 'static + Send + Fn(Value<A>, Value<B>) -> C,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
    C: 'static + Send + Clone,
{
    config: Config,
    left: Box<Signal<A>>,
    right: Box<Signal<B>>,
    f: F,
    initial: SignalType<C>,
}

impl<F, A, B, C> Lift2Signal<F, A, B, C> where
    F: 'static + Send + Fn(Value<A>, Value<B>) -> C,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
    C: 'static + Send + Clone,
{
    pub fn new(config: Config, left: Box<Signal<A>>, right: Box<Signal<B>>, f: F) -> Self {
        let initial = match (left.initial(), right.initial()) {
            (SignalType::Constant(l), SignalType::Constant(r)) => {
                SignalType::Constant(f(Value::Changed(l), Value::Changed(r)))
            }

            (SignalType::Dynamic(l), SignalType::Constant(r)) => {
                SignalType::Dynamic(f(Value::Changed(l), Value::Changed(r)))
            }

            (SignalType::Constant(l), SignalType::Dynamic(r)) => {
                SignalType::Dynamic(f(Value::Changed(l), Value::Changed(r)))
            }

            (SignalType::Dynamic(l), SignalType::Dynamic(r)) => {
                SignalType::Dynamic(f(Value::Changed(l), Value::Changed(r)))
            }
        };

        Lift2Signal {
            config: config,
            left: left,
            right: right,
            f: f,
            initial: initial,
        }
    }
}

impl<F, A, B, C> Signal<C> for Lift2Signal<F, A, B, C> where
    F: 'static + Send + Fn(Value<A>, Value<B>) -> C,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
    C: 'static + Send + Clone,
{
    fn config(&self) -> Config {
        self.config.clone()
    }

    fn initial(&self) -> SignalType<C> {
        self.initial.clone()
    }

    fn push_to(self: Box<Self>, mut target: Option<Box<Push<C>>>) {
        let inner = *self;
        let Lift2Signal {config, left, right, f, initial: _} = inner;

        let (left_tx, left_rx) = sync_channel(config.buffer_size.clone());
        let (right_tx, right_rx) = sync_channel(config.buffer_size.clone());
        let left_initial = left.initial();
        let right_initial = right.initial();

        let mut last_l = match left.initial() {
            SignalType::Constant(l) => l,
            SignalType::Dynamic(l) => {
                thread::spawn(move || {
                    let pusher = InputPusher {
                        tx: left_tx,
                    };
                    left.push_to(Some(Box::new(pusher)));
                });

                l
            },
        };

        let mut last_r = match right.initial().clone() {
            SignalType::Constant(r) => r,
            SignalType::Dynamic(r) => {
                thread::spawn(move || {
                    let pusher = InputPusher {
                        tx: right_tx,
                    };
                    right.push_to(Some(Box::new(pusher)));
                });

                r
            },
        };

        loop {
            let mut any_changed = false;

            let l = match left_initial {
                SignalType::Constant(ref l) => Value::Unchanged(l.clone()),
                SignalType::Dynamic(_) => {
                    match left_rx.recv() {
                        Ok(Event::Changed(l)) => {
                            info!("RUN: Lift2 using changed Left value");
                            any_changed = true;
                            last_l = l.clone();
                            Value::Changed(l)
                        },
                        Ok(Event::Unchanged) => {
                            info!("RUN: Lift2 using cached Left value");
                            Value::Unchanged(last_l.clone())
                        },
                        Ok(Event::Exit) => return,
                        Err(_) => return,
                    }
                }
            };

            let r = match right_initial {
                SignalType::Constant(ref r) => Value::Unchanged(r.clone()),
                SignalType::Dynamic(_) => {
                    match right_rx.recv() {
                        Ok(Event::Changed(r)) => {
                            info!("RUN: Lift2 using changed Right value");
                            any_changed = true;
                            last_r = r.clone();
                            Value::Changed(r)
                        },
                        Ok(Event::Unchanged) => {
                            info!("RUN: Lift2 using cached Right value");
                            Value::Unchanged(last_r.clone())
                        },
                        Ok(Event::Exit) => return,
                        Err(_) => return,
                    }
                }
            };

            let c = if any_changed {
                Event::Changed(f(l,r))
            } else {
                Event::Unchanged
            };

            match target {
                Some(ref mut t) => t.push(c),
                None => {},
            }
        }
    }
}
impl<F, A, B, C> SignalExt<C> for Lift2Signal<F, A, B, C> where
    F: 'static + Send + Fn(Value<A>, Value<B>) -> C,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
    C: 'static + Send + Clone,
{}

// Passed up the 'push_to' chain, finalizes by sending to a channel
struct InputPusher<A> {
    tx: SyncSender<Event<A>>,
}

impl<A> Push<A> for InputPusher<A> where
    A: 'static + Send,
{
    fn push(&mut self, event: Event<A>) {
        debug!("Lift2::InputPusher::push");

        match self.tx.send(event) {
            Err(e) => { debug!("Lift2::InputPusher received error {}", e) },
            _ => {},
        }
    }
}
