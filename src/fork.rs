use std::thread::spawn;
use std::sync::mpsc::*;

use super::{Fork, Branch, Signal, Run, Event};

impl<A> Run for Fork<A> where
    A: 'static + Clone + Send,
{
    fn run(mut self: Box<Self>) {
        spawn(move || {
            loop {
                let received = self.parent.pull();

                match received {
                    Event::Exit => {
                        for sink in self.sink_txs.lock().unwrap().iter() {
                            match sink.send(received.clone()) {
                                // We can't really terminate a child process, so just ignore errors...
                                _ => {},
                            }
                        }

                        return
                    },
                    _ => {
                        for sink in self.sink_txs.lock().unwrap().iter() {
                            match sink.send(received.clone()) {
                                // We can't really terminate a child process, so just ignore errors...
                                _ => {},
                            }
                        }
                    }
                }
            }
        });
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
    fn pull(&mut self) -> Event<A> {
        match self.source_rx.recv() {
            Err(_) => Event::Exit,
            Ok(a) => a,
        }
    }
}
