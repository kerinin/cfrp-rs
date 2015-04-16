//! Provides a set of channels where incoming data on any channel causes a
//! No-Op signal to be sent to all the rest.
//!
//! Used to provide syncronous communication between signals.
//!

use std::thread;
use std::cell::*;
use std::rc::*;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::*;
use std::marker::MarkerTrait;



use super::signal::{Signal};

trait NoOp: Send {
    fn no_op(&self);
}

impl<A> NoOp for Sender<Option<A>> 
where A: 'static + Send
{
    fn no_op(&self) {
        self.send(None);
    }
}

pub struct Coordinator {
    channels: Arc<Mutex<(usize, Vec<Box<NoOp>>)>>,
}

impl Coordinator {
    pub fn channel<A>(&self) -> (Sender<A>, Signal<A>) 
    where A: 'static + Send + Clone,
    {
        // For data coming in
        let (data_tx, data_rx): (Sender<A>, Receiver<A>) = channel();

        // For data going out, type Option<A>
        let (signal_tx, signal_rx): (Sender<Option<A>>, Receiver<Option<A>>) = channel();
        let signal = Signal::new(signal_rx);

        let idx = {
            let &mut (ref mut count, ref mut no_ops) = &mut *self.channels.lock().unwrap();
            let idx = count.clone();

            no_ops.push(Box::new(signal_tx.clone()));
            *count += 1;

            idx
        };

        let channels = self.channels.clone();
        thread::spawn(move || {
            loop {
                match data_rx.recv() {
                    Ok(ref a) => {
                        let (_, ref no_ops) = *channels.lock().unwrap();

                        for (i, no_op) in no_ops.iter().enumerate() {
                            if i == idx {
                                signal_tx.send(Some(a.clone()));
                            } else {
                                no_op.no_op();
                            }
                        }
                    }
                    _ => { return }
                }
            }
        });

        return (data_tx, signal);
    }
}
