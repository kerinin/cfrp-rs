use std::sync::*;
use std::sync::mpsc::*;

use super::{Fork, Branch, Signal, Run, Event};

impl<A> Run for Fork<A> where
    A: 'static + Clone + Send,
{
    fn run(mut self: Box<Self>) {
        loop {
            let received = self.parent.recv();

            for sink in self.sink_txs.lock().unwrap().iter() {
                sink.send(received.clone());
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
    fn recv(&mut self) -> Event<A> {
        match self.source_rx.recv() {
            Err(_) => Event::Exit,
            Ok(a) => a,
        }
    }
}
