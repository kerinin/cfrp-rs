use std::cell::*;
use std::thread;
use std::sync::mpsc::*;

use super::{Coordinator, Signal, Spawn};

pub struct Lift<'a, F, A, B> where
    F: Fn(&A) -> B,
{
    coordinator: &'a Coordinator,
    f: Box<F>,
    source_rx: Receiver<Option<A>>,
    sink_txs: RefCell<Vec<Sender<Option<B>>>>,
}

impl<'a, F, A, B> Lift<'a, F, A, B> where
    F: Fn(&A) -> B,
{
    pub fn new(coordinator: &'a Coordinator, f: Box<F>, source_rx: Receiver<Option<A>>) -> Lift<F, A, B> {
        Lift {
            coordinator: coordinator,
            f: f,
            source_rx: source_rx,
            sink_txs: RefCell::new(Vec::new()),
        }
    }
}

impl<'a, F, A, B> Signal<B> for Lift<'a, F, A, B> where
    F: Fn(&A) -> B,
{
    fn publish_to(&self, tx: Sender<Option<B>>) {
        self.sink_txs.borrow_mut().push(tx);
    }

    fn coordinator(&self) -> &Coordinator {
        &*self.coordinator
    }
}

impl<'a, F, A, B> Spawn for Lift<'a, F, A, B> where
    F: 'static + Send + Fn(&A) -> B,
    A: 'static + Send,
    B: 'static + Send + Eq + Clone,
{
    fn spawn(self: Box<Self>) {
        // NOTE: weird, but required to get access to all the fields simultaneously
        let unboxed = *self;
        let Lift { coordinator: _, f, source_rx, sink_txs } = unboxed;

        let runner = CompiledLift {
            f: f,
            source_rx: source_rx,
            sink_txs: sink_txs.into_inner(),
            last_b: None,
        };
        thread::spawn(move || {
            runner.run();
        });
    }
}


pub struct CompiledLift<F, A, B> where
    F: 'static + Send + Fn(&A) -> B,
    A: 'static + Send,
    B: 'static + Send + Eq + Clone,
{
    f: Box<F>,
    source_rx: Receiver<Option<A>>,
    sink_txs: Vec<Sender<Option<B>>>,
    last_b: Option<B>,
}

impl<F, A, B> CompiledLift<F, A, B> where
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
                    for sink_tx in self.sink_txs.iter() {
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

        for sink_tx in self.sink_txs.iter() {
            match sink_tx.send(value.clone()) {
                Err(_) => { return true }
                _ => {}
            }
        }

        false
    }
}
