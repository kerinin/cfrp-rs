use std::thread;
use std::sync::mpsc::*;

use super::super::{Event, Signal, Push, Lift, Lift2, Fold};

/// The result of a `lift2` operation
///
pub struct Lift2Signal<F, A, B, C> where
    F: 'static + Send + Fn(Option<A>, Option<B>) -> C,
    A: 'static + Send,
    B: 'static + Send,
    C: 'static + Send + Clone,
{
    left: Box<Signal<A>>,
    right: Box<Signal<B>>,
    f: F,
}

impl<F, A, B, C> Lift2Signal<F, A, B, C> where
    F: 'static + Send + Fn(Option<A>, Option<B>) -> C,
    A: 'static + Send,
    B: 'static + Send,
    C: 'static + Send + Clone,
{
    pub fn new(left: Box<Signal<A>>, right: Box<Signal<B>>, f: F) -> Self {
        Lift2Signal {
            left: left,
            right: right,
            f: f,
        }
    }
}

impl<F, A, B, C> Signal<C> for Lift2Signal<F, A, B, C> where
    F: 'static + Send + Fn(Option<A>, Option<B>) -> C,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
    C: 'static + Send + Clone,
{
    fn push_to(self: Box<Self>, target: Option<Box<Push<C>>>) {
        let inner = *self;
        let Lift2Signal {left, right, f} = inner;

        let (left_tx, left_rx) = channel();
        thread::spawn(move || {
            let pusher = InputPusher {
                tx: left_tx,
            };
            debug!("Pushing Lift2::InputPusher left");
            left.push_to(Some(Box::new(pusher)));
        });

        let (right_tx, right_rx) = channel();
        thread::spawn(move || {
            let pusher = InputPusher {
                tx: right_tx,
            };
            debug!("Pushing Lift2::InputPusher right");
            right.push_to(Some(Box::new(pusher)));
        });

        match target {
            Some(mut t) => {
                let mut cached_left = None;
                let mut cached_right = None;

                loop {

                    // NOTE: There's probably a better way of doing this...
                    // Also, we can eliminate _some_ computation by detecting
                    // repeated NoOps and only computing f if there's a change.
                    match (left_rx.recv(), right_rx.recv()) {
                        (Ok(Event::Changed(l)), Ok(Event::Changed(r))) => {
                            cached_left = Some(l.clone());
                            cached_right = Some(r.clone());

                            let c = f(Some(l), Some(r));
                            t.push(Event::Changed(c));
                        }

                        (Ok(Event::Unchanged), Ok(Event::Changed(r))) => {
                            match cached_left {
                                Some(ref l) => {
                                    cached_right = Some(r.clone());

                                    let c = f(Some(l.clone()), Some(r));
                                    t.push(Event::Changed(c));
                                }
                                None => panic!("No cached left value"),
                            }
                        }

                        (Ok(Event::Changed(l)), Ok(Event::Unchanged)) => {
                            match cached_right {
                                Some(ref r) => {
                                    cached_left = Some(l.clone());

                                    let c = f(Some(l), Some(r.clone()));
                                    t.push(Event::Changed(c));
                                }
                                None => panic!("No cached right value"),
                            }
                        }

                        (Ok(Event::Unchanged), Ok(Event::Unchanged)) => {
                            t.push(Event::Unchanged);
                        }

                        (Ok(Event::Changed(l)), Ok(Event::NoOp)) => {
                            cached_left = Some(l.clone());

                            let c = f(Some(l), None);
                            t.push(Event::Changed(c));
                        }

                        (Ok(Event::NoOp), Ok(Event::Changed(r))) => {
                            cached_right = Some(r.clone());

                            let c = f(None, Some(r));
                            t.push(Event::Changed(c));
                        }

                        (Ok(Event::Unchanged), Ok(Event::NoOp)) => {
                            match cached_left {
                                Some(ref l) => {
                                    let c = f(Some(l.clone()), None);
                                    t.push(Event::Changed(c));
                                }
                                None => panic!("No cached right value"),
                            }
                        }

                        (Ok(Event::NoOp), Ok(Event::Unchanged)) => {
                            match cached_right {
                                Some(ref r) => {
                                    let c = f(None, Some(r.clone()));
                                    t.push(Event::Changed(c));
                                }
                                None => panic!("No cached right value"),
                            }
                        }

                        (_, _) => {
                        }
                    }
                }
            },
            None => {
                loop {
                    match (left_rx.recv(), right_rx.recv()) {
                        _ => {}
                    }
                }
            }
        }
    }
}

impl<F, A, B, C> Lift<C> for Lift2Signal<F, A, B, C> where
    F: 'static + Send + Fn(Option<A>, Option<B>) -> C,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
    C: 'static + Send + Clone,
{}
impl<F, A, B, C, D, SD> Lift2<C, D, SD> for Lift2Signal<F, A, B, C> where
    F: 'static + Send + Fn(Option<A>, Option<B>) -> C,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
    C: 'static + Send + Clone,
{}
impl<F, A, B, C> Fold<C> for Lift2Signal<F, A, B, C> where
    F: 'static + Send + Fn(Option<A>, Option<B>) -> C,
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
