use std::sync::mpsc::*;

use super::super::{Event, Signal, SignalType, Push, Lift, Lift2, Fold};

pub struct Channel<A> where
    A: 'static + Send + Clone,
{
    source_rx: Receiver<Event<A>>,
    initial: A,
}

impl<A> Channel<A> where
    A: 'static + Send + Clone,
{
    pub fn new(source_rx: Receiver<Event<A>>, initial: A) -> Channel<A> {
        Channel {
            source_rx: source_rx,
            initial: initial,
        }
    }
}

impl<A> Signal<A> for Channel<A> where
    A: 'static + Send + Clone,
{
    fn initial(&self) -> SignalType<A> {
        SignalType::Dynamic(self.initial.clone())
    }

    fn push_to(self: Box<Self>, target: Option<Box<Push<A>>>) {
        match target {
            Some(mut t) => {
                debug!("Channel::push_to Some");

                loop {
                    match self.source_rx.recv() {
                        Err(e) => {
                            debug!("Channel source_rx received Err {}", e);
                            t.push(Event::Exit);

                            return
                        },
                        Ok(a) => {
                            debug!("Channel source_rx received data");
                            t.push(a);
                        },
                    }
                }
            }
            None => {
                debug!("Channel::push_to None");

                // Just ensuring the channel is drained so we don't get memory leaks
                loop {
                    match self.source_rx.recv() {
                        Err(_) => return,
                        _ => {},
                    }
                }
            },
        }
    }
}

impl<A> Lift<A> for Channel<A> where A: 'static + Send + Clone, {}
impl<A, B, SB> Lift2<A, B, SB> for Channel<A>  where A: 'static + Send + Clone {}
impl<A> Fold<A> for Channel<A> where A: 'static + Send + Clone {}
