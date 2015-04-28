use std::sync::mpsc::*;

use super::super::{Event, Signal, SignalType, Push, Run};

pub struct Async<A> {
    parent: Box<Signal<A>>,
    tx: SyncSender<A>,
}

impl<A> Async<A> {
    pub fn new(parent: Box<Signal<A>>, tx: SyncSender<A>) -> Async<A> {
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

        match parent.initial() {
            SignalType::Constant(_) => return,
            SignalType::Dynamic(_) => {
                parent.push_to(Some(Box::new(AsyncPusher {tx: tx})));
            },
        }
    }
}

struct AsyncPusher<A> {
    tx: SyncSender<A>,
}

impl<A> Push<A> for AsyncPusher<A> where
    A: 'static + Clone + Send,
{
    fn push(&mut self, event: Event<A>) {

        match event {
            Event::Changed(a) => {
                debug!("Async handling Event Changed - pushing to channel");
                match self.tx.send(a) {
                    // We can't really terminate a child process, so just ignore errors...
                    _ => {},
                }
            },
            Event::Unchanged => {
                debug!("Async handling Event Unchanged - doing nothing");
                // No change, so no point in pushing...
            },
            Event::Exit => {
                debug!("Async handling Event Exit");
                // Exit should be propagated to all top-level inputs anyway, so
                // nothing to do here...
            }
        }
    }
}
