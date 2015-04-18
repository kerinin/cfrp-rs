use std::cell::*;
use std::sync::mpsc::*;

use super::*;

pub struct Lift<F, A, B> where
    F: Fn(&A) -> B,
{
    f: Box<F>,
    source_rx: Receiver<Option<A>>,
    sink_txs: RefCell<Vec<Sender<Option<B>>>>,
}

impl<F, A, B> Lift<F, A, B> where
    F: Fn(&A) -> B,
{
    pub fn new(f: Box<F>, source_rx: Receiver<Option<A>>) -> Lift<F, A, B> {
        Lift {
            f: f,
            source_rx: source_rx,
            sink_txs: RefCell::new(Vec::new()),
        }
    }
}

impl<F, A, B> Signal<B> for Lift<F, A, B> where
    F: Fn(&A) -> B,
{
    fn publish_to(&self, tx: Sender<Option<B>>) {
        self.sink_txs.borrow_mut().push(tx);
    }
}

impl<F, A, B> Compile for Lift<F, A, B> where
    F: 'static + Send + Fn(&A) -> B,
    A: 'static + Send,
    B: 'static + Send + Eq + Clone,
{
    fn compile(self) -> Box<Run> {
        let runner = CompiledLift {
            f: self.f,
            source_rx: self.source_rx,
            sink_txs: self.sink_txs,
            last_b: None,
        };
        Box::new(runner)
    }
}


pub struct CompiledLift<F, A, B> {
    f: Box<F>,
    source_rx: Receiver<Option<A>>,
    sink_txs: RefCell<Vec<Sender<Option<B>>>>,
    last_b: Option<B>,
}

impl<F, A, B> Run for CompiledLift<F, A, B> where
    F: Fn(&A) -> B + Send,
    A: Send,
    B: Eq + Clone + Send,
{
    fn run(mut self) {
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
}

impl<F, A, B> CompiledLift<F, A, B> where
    F: Fn(&A) -> B,
    B: Eq + Clone,
{
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
