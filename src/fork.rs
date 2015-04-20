use std::sync::*;
use std::sync::mpsc::*;

use super::{Fork, Branch, Signal, Run};

impl<A> Run for Fork<A> where
    A: 'static + Clone + Send,
{
    fn run(self: Box<Self>) {
        loop {
            match self.parent.recv() {
                Some(a) => {
                    for sink in self.sink_txs.lock().unwrap().iter() {
                        sink.send(Some(a.clone()));
                    }
                },
                _ => {},
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

impl<A> Signal<A> for Branch<A> where
    A: 'static + Send,
{
    fn recv(&self) -> Option<A> {
        match self.source_rx.recv() {
            Err(_) => None,
            Ok(a) => a,
        }
    }
}
