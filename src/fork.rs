use std::sync::*;
use std::sync::mpsc::*;

use super::{Signal, Run};

pub struct Fork<A> where
    A: 'static + Send,
{
    parent: Box<Signal<A> + Send>,
    sink_txs: Arc<Mutex<Vec<Sender<Option<A>>>>>,
}

impl<A> Fork<A> where
    A: 'static + Clone + Send,
{
    pub fn new(parent: Box<Signal<A> + Send>, sink_txs: Arc<Mutex<Vec<Sender<Option<A>>>>>) -> Fork<A> {
        Fork {
            parent: parent,
            sink_txs: sink_txs,
        }
    }
}

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

pub struct Branch<A> where
    A: 'static + Send,
{
    // Arc<T> is send if T: Send + Sync (which mutex is, unconditionally)
    fork_txs: Arc<Mutex<Vec<Sender<Option<A>>>>>,
    source_rx: Receiver<Option<A>>,
}

impl<A> Branch<A> where
    A: 'static + Send,
{
    pub fn new(fork_txs: Arc<Mutex<Vec<Sender<Option<A>>>>>, source_rx: Receiver<Option<A>>) -> Branch<A> {
        Branch {
            fork_txs: fork_txs,
            source_rx: source_rx,
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
