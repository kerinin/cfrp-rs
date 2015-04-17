use std::thread;
use std::cell::*;
use std::marker::*;
use std::sync::mpsc::*;

use super::coordinator2::{Coordinator};
use super::signal2::{Signal};

pub trait Transform {
    fn run(self);
}

pub struct Lift<'a, F, A, B> 
where A: 'static,
{
    pub f: F,
    pub source_rx: Receiver<Option<A>>,
    pub sink_tx: Sender<Option<B>>,
    // pub sink: Signal<'a, B>,
}

impl<'a, F, A, B> Lift<'a, F, A, B> 
where F: 'static + Fn(&A) -> B,
    A: 'static + Clone + Send,
    B: 'static + Clone + Send + Eq,
{
    fn send_if_changed(&self, a: &A) -> bool {
        let b = Some((self.f)(a));

        // let value = if self.last_b == b {
        //     None
        // } else {
        //     self.last_b = b.clone();
        //     b
        // };

        // match self.sink_tx.send(value) {
        //     Err(_) => { true }
        //     _ => { false }
        // }
        true
    }
}

impl<'a, F, A, B> Transform for Lift<'a, F, A, B>
where F: 'static + Fn(&A) -> B,
    A: 'static + Clone + Send,
    B: 'static + Clone + Send + Eq,
{
    fn run(self) {
        loop {
            match self.source_rx.recv() {
                // Signal value changed
                Ok(Some(ref a)) => {
                    if self.send_if_changed(a) { break }
                }
                
                // No change from previous - send it along!
                Ok(None) => {
                    match self.sink_tx.send(None) {
                        Err(_) => { break }
                        _ => {}
                    }
                }

                // Receiver closed, time to pack up & go home
                _ => { break; }
            }
        }
    }
}
