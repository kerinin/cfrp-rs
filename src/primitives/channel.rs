use std::sync::mpsc::*;

use super::super::{Event, Signal, SignalType, Push};

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

/*
#[cfg(test)] 
mod test {
    extern crate env_logger;

    use std::sync::mpsc::*;

    use super::super::topology::{Topology, Builder};

    // env_logger::init().unwrap();

    #[test]
    fn channel_runs() {
        let b = Builder::new();
        b.value(1usize);

        Topology::new(b).run();

        assert!(true);
    }

    #[test]
    fn channel_receives_data() {
        let(tx, rx): (SyncSender<usize>, Receiver<usize>) = sync_channel(0);

        let b = Builder::new();
        let input = b.listen(0, rx);

        let t = Topology::new(b);
        t.run();

        println!("Ran topology");
        tx.send(0).unwrap();

        assert!(true);
    }
}
*/
