use std::sync::*;
use std::sync::mpsc::*;

use super::{Fork, Branch, Signal, Run, Push, Event};

// Fork is the incoming portion of a fork.  It is pushed data from upstream and
// clones it across a (possibly empty) set of child branches
//
impl<A> Run for Fork<A> where
    A: 'static + Clone + Send,
{
    fn run(self: Box<Self>) {
        let inner = *self;
        let Fork { parent, sink_txs } = inner;

        parent.push_to(
            Box::new(
                ForkPusher {
                    sink_txs: sink_txs,
                }
            )
        )
    }
}

struct ForkPusher<A> {
    sink_txs: Arc<Mutex<Vec<Sender<Event<A>>>>>,
}

impl<A> Push<A> for ForkPusher<A> where
    A: 'static + Clone + Send,
{
    fn push(&mut self, event: Event<A>) {
        for sink_tx in self.sink_txs.lock().unwrap().iter() {
            match sink_tx.send(event.clone()) {
                // We can't really terminate a child process, so just ignore errors...
                _ => {},
            }
        }
    }
}

// Branch is the outgoig portion of a fork.  It spawns waits for incoming data 
// from it's parent fork and pushes it to its children
//
impl<A> Signal<A> for Branch<A> where
    A: 'static + Send,
{
    fn push_to(self: Box<Self>, mut target: Box<Push<A>>) {
        loop {
            match self.source_rx.recv() {
                Ok(event) => target.push(event),
                Err(_) => return,
            }
        }
    }
}

impl<A> Clone for Branch<A> where
    A: 'static + Send,
{
    fn clone(&self) -> Branch<A> {
        let (tx, rx) = channel();
        self.fork_txs.lock().unwrap().push(tx);
        Branch { fork_txs: self.fork_txs.clone(), source_rx: rx }
    }
}
