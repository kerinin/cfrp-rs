use std::thread;
use std::cell::*;
use std::sync::mpsc::*;

use super::coordinator2::{Coordinator};

pub struct Signal<'a, T> {
    pub coordinator: &'a Coordinator,
    rx: Receiver<Option<T>>,
    txs: RefCell<Vec<Sender<Option<T>>>>,
}

impl<'a, T: 'static + Clone + Send> Signal<'a, T> {
    pub fn new(coordinator: &'a Coordinator, rx: Receiver<Option<T>>) -> Signal<'a, T> {
        Signal { coordinator: coordinator, rx: rx, txs: RefCell::new(Vec::new()) }
    }
    
    pub fn publish_to(&self, tx: Sender<Option<T>>) {
        self.txs.borrow_mut().push(tx);
    }

    pub fn run(self) {
        let txs = self.txs.into_inner();
        loop {
            match self.rx.recv() {
                Ok(t) => {
                    for tx in txs.iter() {
                        match tx.send(t.clone()) {
                            Ok(_) => {},
                            Err(_) => { break; },
                        }
                    }
                }
                Err(_) => { break; }
            }
        }
    }
}
