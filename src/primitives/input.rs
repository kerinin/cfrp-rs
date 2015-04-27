use std::sync::*;
use std::sync::mpsc::*;

use super::super::{Event};

pub trait NoOp: Send {
    fn send_no_change(&self) -> bool;
    fn send_exit(&self);
}

pub trait RunInput: Send {
    fn run(mut self: Box<Self>, usize, Arc<Mutex<Vec<Box<NoOp>>>>);
    fn boxed_no_op(&self) -> Box<NoOp>;
}

pub struct ReceiverInput<A> {
    rx: Receiver<A>,
    tx: SyncSender<Event<A>>,
}

impl<A> ReceiverInput<A> {
    pub fn new(rx: Receiver<A>, tx: SyncSender<Event<A>>) -> ReceiverInput<A> {
        ReceiverInput {
            rx: rx,
            tx: tx,
        }
    }
}

impl<A> RunInput for ReceiverInput<A> where
    A: 'static + Send + Clone,
{
    fn boxed_no_op(&self) -> Box<NoOp> {
        Box::new(self.tx.clone())
    }

    fn run(self: Box<Self>, idx: usize, txs: Arc<Mutex<Vec<Box<NoOp>>>>) {
        debug!("SETUP: running ReceiverInput");
        let inner = *self;
        let ReceiverInput {rx, tx} = inner;

        loop {
            match rx.recv() {
                Ok(ref a) => {
                    info!("RUN: ReceiverInput received data, sending");
                    for (i, no_op_tx) in txs.lock().unwrap().iter().enumerate() {
                        if i == idx {
                            match tx.send(Event::Changed(a.clone())) {
                                Err(_) => return,
                                _ => {},
                            }
                        } else {
                            if no_op_tx.send_no_change() { return }
                        }
                    }
                },
                Err(e) => {
                    info!("RUN: ReceiverInput sending error {}, exiting", e);
                    for no_op_tx in txs.lock().unwrap().iter() {
                        no_op_tx.send_exit();
                    }
                    return
                },
            }
        }
    }
}


impl<A> NoOp for SyncSender<Event<A>> where
A: Send
{
    fn send_no_change(&self) -> bool {
        info!("RUN: SyncSender sending Unchanged");
        match self.send(Event::Unchanged) {
            Err(_) => true,
            _ => false,
        }
    }

    fn send_exit(&self) {
        info!("RUN: SyncSender sending Exit");
        match self.send(Event::Exit) {
            _ => {}
        }
    }
}
