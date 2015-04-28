use std::sync::mpsc::*;

use super::super::{Event, Signal, SignalExt, SignalType, Push, Config};

pub struct Channel<A> where
    A: 'static + Send + Clone,
{
    config: Config,
    source_rx: Receiver<Event<A>>,
    initial: A,
}

impl<A> Channel<A> where
    A: 'static + Send + Clone,
{
    pub fn new(config: Config, source_rx: Receiver<Event<A>>, initial: A) -> Channel<A> {
        Channel {
            config: config,
            source_rx: source_rx,
            initial: initial,
        }
    }
}

impl<A> Signal<A> for Channel<A> where
    A: 'static + Send + Clone,
{
    fn config(&self) -> Config {
        self.config.clone()
    }

    fn initial(&self) -> SignalType<A> {
        SignalType::Dynamic(self.initial.clone())
    }

    fn push_to(self: Box<Self>, target: Option<Box<Push<A>>>) {
        match target {
            Some(mut t) => {
                debug!("SETUP: Sending to Some");
                loop {
                    match self.source_rx.recv() {
                        Err(e) => {
                            info!("RUN: Channel source_rx received Err {}, exiting", e);
                            t.push(Event::Exit);

                            return
                        },
                        Ok(a) => {
                            info!("RUN: Channel source_rx received data, pushing");
                            t.push(a);
                        },
                    }
                }
            }
            None => {
                debug!("SETUP: Sending to None");
                // Just ensuring the channel is drained so we don't get memory leaks
                loop {
                    match self.source_rx.recv() {
                        Err(e) => {
                            info!("RUN: Channel source_rx received Err {} with no target, exiting", e);
                            return
                        },
                        _ => {
                            info!("RUN: source_rx received data, but no target");
                        },
                    }
                }
            },
        }
    }
}
impl<A> SignalExt<A> for Channel<A> where A: 'static + Send + Clone {}
