use std::thread;
use std::sync::mpsc::*;

use super::super::{Event, Signal, SignalExt, SignalType, Push};

/// The result of a `lift2` operation
///
pub struct Lift2Signal<F, A, B, C> where
    F: 'static + Send + Fn(A, B) -> C,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
    C: 'static + Send + Clone,
{
    left: Box<Signal<A>>,
    right: Box<Signal<B>>,
    f: F,
    initial: SignalType<C>,
}

impl<F, A, B, C> Lift2Signal<F, A, B, C> where
    F: 'static + Send + Fn(A, B) -> C,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
    C: 'static + Send + Clone,
{
    pub fn new(left: Box<Signal<A>>, right: Box<Signal<B>>, f: F) -> Self {
        let initial = match (left.initial(), right.initial()) {
            (SignalType::Constant(l), SignalType::Constant(r)) => {
                SignalType::Constant(f(l, r))
            }

            (SignalType::Dynamic(l), SignalType::Constant(r)) => {
                SignalType::Dynamic(f(l, r))
            }

            (SignalType::Constant(l), SignalType::Dynamic(r)) => {
                SignalType::Dynamic(f(l, r))
            }

            (SignalType::Dynamic(l), SignalType::Dynamic(r)) => {
                SignalType::Dynamic(f(l, r))
            }
        };

        Lift2Signal {
            left: left,
            right: right,
            f: f,
            initial: initial,
        }
    }
}

impl<F, A, B, C> Signal<C> for Lift2Signal<F, A, B, C> where
    F: 'static + Send + Fn(A, B) -> C,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
    C: 'static + Send + Clone,
{
    fn initial(&self) -> SignalType<C> {
        self.initial.clone()
    }

    fn push_to(self: Box<Self>, mut target: Option<Box<Push<C>>>) {
        let inner = *self;
        let Lift2Signal {left, right, f, initial: _} = inner;

        let (left_tx, left_rx) = channel();
        let (right_tx, right_rx) = channel();
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
                SignalType::Constant(ref l) => l.clone(),
                SignalType::Dynamic(_) => {
                    match left_rx.recv() {
                        Ok(Event::Changed(l)) => {
                            info!("RUN: Lift2 using changed Left value");
                            any_changed = true;
                            last_l = l.clone();
                            l
                        },
                        Ok(Event::Unchanged) => {
                            info!("RUN: Lift2 using cached Left value");
                            last_l.clone()
                        },
                        Ok(Event::Exit) => return,
                        Err(_) => return,
                    }
                }
            };

            let r = match right_initial {
                SignalType::Constant(ref r) => r.clone(),
                SignalType::Dynamic(_) => {
                    match right_rx.recv() {
                        Ok(Event::Changed(r)) => {
                            info!("RUN: Lift2 using changed Right value");
                            any_changed = true;
                            last_r = r.clone();
                            r
                        },
                        Ok(Event::Unchanged) => {
                            info!("RUN: Lift2 using cached Right value");
                            last_r.clone()
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
    F: 'static + Send + Fn(A, B) -> C,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
    C: 'static + Send + Clone,
{}

// Passed up the 'push_to' chain, finalizes by sending to a channel
struct InputPusher<A> {
    tx: Sender<Event<A>>,
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

/*
#[cfg(test)] 
mod test {
    extern crate env_logger;

    use std::sync::mpsc::*;

    use super::super::super::Signal;
    use super::super::channel::Channel;
    use super::super::value::Value;
    use super::super::lift2::Lift2Signal;

    // env_logger::init().unwrap();

    #[test]
    fn lift2_constructs_from_values() {
        let v1 = Value::new(0);
        let v2 = Value::new(0);
        let l = Lift2Signal::new(Box::new(v1), Box::new(v2), |i, j| -> usize { i + j }, 0);

        assert!(true);
    }

    #[test]
    fn lift2_constructs_from_channel_and_value() {
        let (tx, rx) = channel();
        let c = Channel::new(rx, 0);
        let v = Value::new(0);
        let l = Lift2Signal::new(Box::new(c), Box::new(v), |i,j| { i + j }, 0);

        assert!(true);
    }

    #[test]
    fn lift2_constructs_from_channels() {
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();
        let c1 = Channel::new(rx1, 0);
        let c2 = Channel::new(rx2, 0);

        let l = Lift2Signal::new(Box::new(c1), Box::new(c2), |i,j| { i + j }, 0);

        assert!(true);
    }

    #[test]
    fn lift2_lifts_from_channels() {
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();
        let (tx3, rx3) = channel();
        let c1: Box<Signal<usize>> = Box::new(Channel::new(rx1, 0usize));
        let c2: Box<Signal<usize>> = Box::new(Channel::new(rx2, 0usize));
        let c3: Box<Signal<usize>> = Box::new(Channel::new(rx3, 0usize));

        // let l: Box<Signal<usize>> = Box::new(Lift2Signal::new(c1, c2, |i,j| -> usize { i + j }, 0));
        // let l2: Box<Signal<usize>> = l.lift2(c3, |i,j| -> usize { i + j });
        //
        assert!(true);
    }
}
*/
