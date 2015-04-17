use std::thread::*;
use std::cell::*;
use std::sync::mpsc::*;

use super::signal::Signal;

pub trait RunChannel: Send {
    fn run(self, Vec<Box<NoOp>>);
}


pub struct Channel<A> {
    idx: usize,
    source_rx: Receiver<A>,
    sink_txs: RefCell<Vec<Sender<Option<A>>>>,
}

impl<A> Signal<A> for Channel<A> {
    fn publish_to(&self, tx: Sender<Option<A>>) {
        self.sink_txs.borrow_mut().push(tx);
    }
}

impl<A> RunChannel for Channel<A>
where A: 'static + Send + Clone,
{
    fn run(self, no_ops: Vec<Box<NoOp>>) {
        loop {
            match self.source_rx.recv() {
                Ok(ref a) => {
                    for (i, no_op) in no_ops.iter().enumerate() {
                        if i == self.idx {
                            for sink_tx in self.sink_txs.borrow().iter() {
                                sink_tx.send(Some(a.clone()));
                            }
                        } else {
                            no_op.no_op();
                        }
                    }
                }
                _ => { return }
            }
        }
    }
}


trait NoOp: Send {
    fn no_op(&self);
    fn boxed_clone(&self) -> Box<NoOp>;
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


pub struct Coordinator {
    no_ops: RefCell<Vec<Box<NoOp>>>,
    channels: RefCell<Vec<Box<RunChannel>>>,
}

impl Coordinator {
    /*
    pub fn channel<'a, A>(&'a self) -> (Sender<A>, Signal<A>) 
        where A: 'static + Send + Clone,
    {
        // For data coming in
        let (data_tx, data_rx): (Sender<A>, Receiver<A>) = channel();

        // For data going out, type Option<A>
        let (signal_tx, signal_rx): (Sender<Option<A>>, Receiver<Option<A>>) = channel();
        let signal = Signal::new(self, signal_rx);

        {
            let idx = self.no_ops.borrow().len();

            self.no_ops.borrow_mut().push(Box::new(signal_tx.clone()));

            let channel = Channel {
                idx: idx,
                source_rx: data_rx,
                signal_tx: signal_tx.clone(),
            };

            self.channels.borrow_mut().push(Box::new(channel));
        }

        return (data_tx, signal);
    }
    */

    pub fn run(self) {
        /*
        // Consume self and spawn some stuff
        for channel in self.channels.into_inner().into_iter() {
            let no_ops = self.no_ops.borrow()
                .iter().map(|i| i.boxed_clone()).collect();

            spawn(move || {
                channel.run(no_ops);
            });
        }

        // NOTE: Return a control channel or something?
        */
    }
}
