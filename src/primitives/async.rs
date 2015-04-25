use std::sync::mpsc::*;

use super::super::{Event, Signal, Push};
use super::fork::Run;

pub struct Async<A> {
    parent: Box<Signal<A>>,
    tx: Sender<A>,
}

impl<A> Async<A> {
    pub fn new(parent: Box<Signal<A>>, tx: Sender<A>) -> Async<A> {
        Async {
            parent: parent,
            tx: tx,
        }
    }
}

impl<A> Run for Async<A> where
    A: 'static + Send + Clone
{
    fn run(self: Box<Self>) {
        debug!("Async::run");

        let inner = *self;
        let Async { parent, tx } = inner;

        parent.push_to(Some(Box::new(AsyncPusher {tx: tx})));
    }
}

pub struct AsyncPusher<A> {
    tx: Sender<A>,
}

impl<A> Push<A> for AsyncPusher<A> where
    A: 'static + Clone + Send,
{
    fn push(&mut self, event: Event<A>) {
        debug!("Async handling Event");

        match event {
            Event::Changed(a) => {
                match self.tx.send(a) {
                    // We can't really terminate a child process, so just ignore errors...
                    _ => {},
                }
            },
            Event::Unchanged => {
                // NOTE: Send cached value
            },
            _ => {},
        }
    }
}
