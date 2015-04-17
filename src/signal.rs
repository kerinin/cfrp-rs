use std::cell::*;
use std::sync::mpsc::*;

use super::coordinator::{Coordinator};

pub trait Run {
    fn run(self);
}


pub trait Signal<A> {
    fn publish_to(&self, Sender<Option<A>>);
}


pub struct Lift<'a, F, A, B> {
    coordinator: &'a Coordinator,
    f: F,
    source_rx: Receiver<Option<A>>,
    sink_txs: RefCell<Vec<Sender<Option<B>>>>,
}

impl<'a, F, A, B> Signal<B> for Lift<'a, F, A, B> {
    fn publish_to(&self, tx: Sender<Option<B>>) {
        self.sink_txs.borrow_mut().push(tx);
    }
}

impl<'a, F, A, B> Run for Lift<'a, F, A, B> where
    F: Send + Fn(&A) -> B,
    A: Send,
    B: Send + Eq + Clone,
{
    fn run(self) {
        let mut runner = LiftRunner {
            f: self.f,
            source_rx: self.source_rx,
            sink_txs: self.sink_txs,
            last_b: None,
        };
        runner.run()
    }
}

pub struct LiftRunner<F, A, B> {
    f: F,
    source_rx: Receiver<Option<A>>,
    sink_txs: RefCell<Vec<Sender<Option<B>>>>,
    last_b: Option<B>,
}

impl<F, A, B> LiftRunner<F, A, B> where
    F: Send + Fn(&A) -> B,
    A: Send,
    B: Send + Eq + Clone,
{
    pub fn run(mut self) {
        loop {
            match self.source_rx.recv() {
                // Signal value changed
                Ok(Some(ref a)) => {
                    if self.send_if_changed(a) { break }
                }

                // No change from previous - send it along!
                Ok(None) => {
                    for sink_tx in self.sink_txs.borrow().iter() {
                        match sink_tx.send(None) {
                            Err(_) => { break }
                            _ => {}
                        }
                    }
                }

                // Receiver closed, time to pack up & go home
                _ => { break; }
            }
        }
    }

    fn send_if_changed(&mut self, a: &A) -> bool {
        let b = Some((self.f)(a));

        let value = if self.last_b == b {
            None
        } else {
            self.last_b = b.clone();
            b
        };

        for sink_tx in self.sink_txs.borrow().iter() {
            match sink_tx.send(value.clone()) {
                Err(_) => { return true }
                _ => {}
            }
        }

        false
    }
}
