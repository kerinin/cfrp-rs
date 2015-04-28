use rand;
use std::sync::*;
use std::sync::mpsc::*;

use super::super::Event;

pub trait NoOp: Send {
    fn send_no_change(&mut self) -> bool;
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
                    for (i, mut no_op_tx) in txs.lock().unwrap().iter_mut().enumerate() {
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
    fn send_no_change(&mut self) -> bool {
        info!("RUN: Sender sending Unchanged");
        match self.send(Event::Unchanged) {
            Err(_) => true,
            _ => false,
        }
    }

    fn send_exit(&self) {
        info!("RUN: Sender sending Exit");
        match self.send(Event::Exit) {
            _ => {}
        }
    }
}

#[derive(Clone)]
pub struct TickInput<A> where
A: Send + Clone,
{
    initial: A,
    tx: SyncSender<Event<A>>,
}

impl<A> TickInput<A> where 
    A: Send + Clone
{
    pub fn new(v: A, tx: SyncSender<Event<A>>) -> Self {
        TickInput { initial: v, tx: tx}
    }
}

impl<A> RunInput for TickInput<A> where
A: 'static + Send + Clone,
{
    fn run(self: Box<Self>, _: usize, _: Arc<Mutex<Vec<Box<NoOp>>>>) {
        // Nothing to do here - all the work is done on NoOp
    }

    fn boxed_no_op(&self) -> Box<NoOp> {
        Box::new(self.clone())
    }
}

impl<A> NoOp for TickInput<A> where
A: Send + Clone
{
    fn send_no_change(&mut self) -> bool {
        info!("RUN: Tick sending value");
        match self.tx.send(Event::Changed(self.initial.clone())) {
            Err(_) => true,
            _ => false,
        }
    }

    fn send_exit(&self) {
        info!("RUN: Tick sending Exit");
        match self.tx.send(Event::Exit) {
            _ => {}
        }
    }
}

#[derive(Clone)]
pub struct RngInput<R, A> where
R: rand::Rng + Clone + Send,
A: Send + Clone + rand::Rand,
{
    rng: R,
    tx: SyncSender<Event<A>>,
}

impl<R, A> RngInput<R, A> where 
R: rand::Rng + Clone + Send,
A: Send + Clone + rand::Rand,
{
    pub fn new(rng: R, tx: SyncSender<Event<A>>) -> Self {
        RngInput { rng: rng, tx: tx}
    }
}

impl<R, A> RunInput for RngInput<R, A> where
R: 'static + rand::Rng + Clone + Send,
A: 'static + Send + Clone + rand::Rand,
{
    fn run(self: Box<Self>, _: usize, _: Arc<Mutex<Vec<Box<NoOp>>>>) {
        // Nothing to do here - all the work is done on NoOp
    }

    fn boxed_no_op(&self) -> Box<NoOp> {
        Box::new(self.clone())
    }
}

impl<R, A> NoOp for RngInput<R, A> where
R: rand::Rng + Clone + Send,
A: Send + Clone + rand::Rand,
{
    fn send_no_change(&mut self) -> bool {
        info!("RUN: Rng sending value");
        let a = self.rng.gen();
        match self.tx.send(Event::Changed(a)) {
            Err(_) => true,
            _ => false,
        }
    }

    fn send_exit(&self) {
        info!("RUN: Rng sending Exit");
        match self.tx.send(Event::Exit) {
            _ => {}
        }
    }
}
