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



use super::signal2::{Signal};

trait NoOp: Send {
    fn no_op(&self);
    fn boxed_clone(&self) -> Box<NoOp>;
}

trait Channel {
    fn spawn(self: Box<Self>, Vec<Box<NoOp>>);
}

impl<A> NoOp for Sender<Option<A>> 
where A: 'static + Send
{
    fn no_op(&self) {
        self.send(None);
    }
    fn boxed_clone(&self) -> Box<NoOp> {
        Box::new(self.clone())
    }
}

struct ChannelData<A> {
    idx: usize,
    data_rx: Receiver<A>,
    signal_tx: Sender<Option<A>>,
}

impl<A> Channel for ChannelData<A> 
where A: 'static + Send + Clone,
{
    fn spawn(self: Box<Self>, no_ops: Vec<Box<NoOp>>) {
        thread::spawn(move || {
            loop {
                match self.data_rx.recv() {
                    Ok(ref a) => {
                        for (i, no_op) in no_ops.iter().enumerate() {
                            if i == self.idx {
                                self.signal_tx.send(Some(a.clone()));
                            } else {
                                no_op.no_op();
                            }
                        }
                    }
                    _ => { return }
                }
            }
        });
    }
}

pub struct Coordinator {
    // Mutex to coordinate access to both fields, RefCell so we can consume the
    // Channels on spawn
    no_ops: Mutex<Vec<Box<NoOp>>>,
    channels: RefCell<Vec<Box<Channel>>>,
}

impl Coordinator {
    pub fn channel<'a, A>(&'a self) -> (Sender<A>, Signal<'a, A>) 
    where A: 'static + Send + Clone,
    {
        // For data coming in
        let (data_tx, data_rx): (Sender<A>, Receiver<A>) = channel();

        // For data going out, type Option<A>
        let (signal_tx, signal_rx): (Sender<Option<A>>, Receiver<Option<A>>) = channel();
        let signal = Signal::publish(self, signal_rx);

        {
            let ref mut no_ops = &mut *self.no_ops.lock().unwrap();
            let idx = no_ops.len();

            no_ops.push(Box::new(signal_tx.clone()));

            let channel_data = ChannelData {
                idx: idx,
                data_rx: data_rx,
                signal_tx: signal_tx.clone(),
            };

            self.channels.borrow_mut().push(Box::new(channel_data));
        }

        return (data_tx, signal);
    }

    pub fn spawn(self) {
        // Clone it up so we don't have to keep a pointer to no_ops
        let cloned_no_ops: Vec<Box<NoOp>> = self.no_ops.lock().unwrap()
            .iter().map(|i| i.boxed_clone()).collect();

        // Consume self and spawn some stuff
        for channel in self.channels.into_inner().into_iter() {
            channel.spawn(cloned_no_ops.iter().map(|i| i.boxed_clone()).collect());
        }


        // NOTE: Return a control channel or something?
    }
}
