use std::thread;
use std::sync::mpsc::*;

use super::super::{Event, Signal, SignalType, Push};

/// The result of a `lift2` operation
///
pub struct Lift2Signal<F, A, B, C> where
    F: 'static + Send + Fn(A, B) -> C,
    A: 'static + Send,
    B: 'static + Send,
    C: 'static + Send + Clone,
{
    left: Box<Signal<A>>,
    right: Box<Signal<B>>,
    f: F,
    initial: C,
}

impl<F, A, B, C> Lift2Signal<F, A, B, C> where
    F: 'static + Send + Fn(A, B) -> C,
    A: 'static + Send,
    B: 'static + Send,
    C: 'static + Send + Clone,
{
    pub fn new(left: Box<Signal<A>>, right: Box<Signal<B>>, f: F, initial: C) -> Self {
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
        SignalType::Dynamic(self.initial.clone())
    }

    fn push_to(self: Box<Self>, target: Option<Box<Push<C>>>) {
        let inner = *self;
        let Lift2Signal {left, right, f, initial: _} = inner;

        debug!("SETUP: spawning listener left");
        let (left_tx, left_rx) = sync_channel(0);
        thread::spawn(move || {
            let pusher = InputPusher {
                tx: left_tx,
            };
            left.push_to(Some(Box::new(pusher)));
        });

        debug!("SETUP: spawning listener right");
        let (right_tx, right_rx) = sync_channel(0);
        thread::spawn(move || {
            let pusher = InputPusher {
                tx: right_tx,
            };
            right.push_to(Some(Box::new(pusher)));
        });

        let mut cached_left = None;
        let mut cached_right = None;
        match target {
            Some(mut t) => {
                debug!("SETUP: pushing to target Some");
                loop {

                    // NOTE: There's probably a better way of doing this...
                    // Also, we can eliminate _some_ computation by detecting
                    // repeated NoOps and only computing f if there's a change.
                    match (left_rx.recv(), right_rx.recv()) {
                        (Ok(Event::Changed(l)), Ok(Event::Changed(r))) => {
                            debug!("Lift2Pusher handling Event::Changed/Event::Changed");
                            cached_left = Some(l.clone());
                            cached_right = Some(r.clone());

                            let c = f(l, r);
                            t.push(Event::Changed(c));
                        }

                        (Ok(Event::Unchanged), Ok(Event::Changed(r))) => {
                            debug!("Lift2Pusher handling Event::Unchanged/Event::Changed");
                            match cached_left {
                                Some(ref l) => {
                                    cached_right = Some(r.clone());

                                    let c = f(l.clone(), r);
                                    t.push(Event::Changed(c));
                                }
                                None => panic!("No cached left value"),
                            }
                        }
                        (Ok(Event::Changed(l)), Ok(Event::Unchanged)) => {
                            debug!("Lift2Pusher handling Event::Changed/Event::Unchanged");
                            match cached_right {
                                Some(ref r) => {
                                    cached_left = Some(l.clone());

                                    let c = f(l, r.clone());
                                    t.push(Event::Changed(c));
                                }
                                None => panic!("No cached right value"),
                            }
                        }

                        (Ok(Event::Unchanged), Ok(Event::Unchanged)) => {
                            debug!("Lift2Pusher handling Event::Unchanged/Event::Unchanged");
                            t.push(Event::Unchanged);
                        }

                        (Ok(Event::Exit), _) => { debug!("Lift2Pusher handling Event::Exit"); t.push(Event::Exit); return }
                        (_, Ok(Event::Exit)) => { debug!("Lift2Pusher handling Event::Exit"); t.push(Event::Exit); return }
                        (Err(_), _) => { debug!("Lift2Pusher handling closed channel"); t.push(Event::Exit); return }
                        (_, Err(_)) => { debug!("Lift2Pusher handling closed channel"); t.push(Event::Exit); return }
                    }
                }
            },
            None => {
                debug!("SETUP: pushing to target None");
                loop {
                    match (left_rx.recv(), right_rx.recv()) {
                        (Ok(Event::Changed(l)), Ok(Event::Changed(r))) => {
                            debug!("Lift2Pusher handling Event::Changed/Event::Changed");
                            cached_left = Some(l.clone());
                            cached_right = Some(r.clone());

                            f(l, r);
                        }

                        (Ok(Event::Unchanged), Ok(Event::Changed(r))) => {
                            debug!("Lift2Pusher handling Event::Unchanged/Event::Changed");
                            match cached_left {
                                Some(ref l) => {
                                    cached_right = Some(r.clone());

                                    f(l.clone(), r);
                                }
                                None => panic!("No cached left value"),
                            }
                        }
                        (Ok(Event::Changed(l)), Ok(Event::Unchanged)) => {
                            debug!("Lift2Pusher handling Event::Changed/Event::Unchanged");
                            match cached_right {
                                Some(ref r) => {
                                    cached_left = Some(l.clone());

                                    f(l, r.clone());
                                }
                                None => panic!("No cached right value"),
                            }
                        }

                        (Ok(Event::Unchanged), Ok(Event::Unchanged)) => {
                            debug!("Lift2Pusher handling Event::Unchanged/Event::Unchanged");
                        }

                        (Ok(Event::Exit), _) => { debug!("Lift2Pusher handling Event::Exit"); return }
                        (_, Ok(Event::Exit)) => { debug!("Lift2Pusher handling Event::Exit"); return }
                        (Err(_), _) => { debug!("Lift2Pusher handling closed channel"); return }
                        (_, Err(_)) => { debug!("Lift2Pusher handling closed channel"); return }
                    }
                }
            }
        }
    }
}

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
