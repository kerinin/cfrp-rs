use std::sync::mpsc::*;

use super::{Signal};

pub struct Channel<A> where
    A: 'static + Send,
{
    source_rx: Receiver<Option<A>>,
}

impl<A> Channel<A> where
    A: 'static + Send,
{
    pub fn new(source_rx: Receiver<Option<A>>) -> Channel<A> {
        Channel {
            source_rx: source_rx,
        }
    }
}

impl<A> Signal<A> for Channel<A> where
    A: 'static + Send,
{
    fn recv(&self) -> Option<A> {
        match self.source_rx.recv() {
            Err(_) => None,
            Ok(a) => a,
        }
    }
}
