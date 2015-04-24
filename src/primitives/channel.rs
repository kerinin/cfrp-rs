use std::sync::mpsc::*;

use super::*;

pub struct Channel<A> where
    A: 'static + Send,
{
    source_rx: Receiver<Event<A>>,
}

impl<A> Channel<A> where
    A: 'static + Send,
{
    pub fn new(source_rx: Receiver<Event<A>>) -> Channel<A> {
        Channel {
            source_rx: source_rx,
        }
    }
}

impl<A> InternalSignal<A> for Channel<A> where
    A: 'static + Send,
{
    fn push_to(self: Box<Self>, target: Option<Box<Push<A>>>) {
        match target {
            Some(mut t) => {
                println!("Channel::push_to Some");

                loop {
                    match self.source_rx.recv() {
                        Err(e) => {
                            println!("Channel source_rx received Err {}", e);
                            t.push(Event::Exit);

                            return
                        },
                        Ok(a) => {
                            println!("Channel source_rx received data");
                            t.push(a);
                        },
                    }
                }
            }
            None => {
                println!("Channel::push_to None");

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
