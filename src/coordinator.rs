//! Provides a set of channels where incoming data on any channel causes a
//! No-Op signal to be sent to all the rest.
//!
//! Used to provide syncronous communication between signals.
//!

use std::thread;
use std::cell::*;
use std::rc::*;
use std::sync::mpsc::*;

use super::signal::{Signal};

/// Multiplex data and incoming channels
///
enum Either<T> {
    Data(T),
    Channel(Box<NoOp>),
}

/// Types which can emit No-Op signals
///
/// Implements a custom clone method to ensure the cloned value is sized.  For
/// some reason adding the Clone trait to NoOp makes it not object-safe...
///
trait NoOp: Send {
    fn no_op(&self);
    fn boxed_clone(&self) -> Box<NoOp>;
}

/// Senders capable of sending No-Op trait objects
///
/// This trait allows channels multiplexing different types to be added to the 
/// same vector.
///
trait NoOpTx {
    fn send(&self, Box<NoOp>);
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

impl<T> NoOpTx for Sender<Either<T>> {
    fn send(&self, no_op: Box<NoOp>) {
        self.send(Either::Channel(no_op));
    }
}

/// # Concurrent FRP signal graph coordinator
///
/// All events processed by a signal graph must originate from a Coordinator.  The
/// coordinator is responsible for ensuring that incoming messages to a given
/// channel are mirrored across all channels as No-Op events.  This ensures that 
/// any signal entering the graph will be delivered to every node in the graph.
/// 
/// NOTE: Need to add checks against mixed topologies
///
/// # Examples:
///
/// ```
/// // Create a new coordinator
/// let mut coordinator = Coordinator::new();
///
/// // Create a sender and signal.  The sender can be used to inject data into
/// // the topology, and the signal can be used to build the topology itself.
/// let (tx, signal) = coordinator.channel();
///
/// lift( signal, |i| { i + 1usize });
///
/// // Note that once data has been sent to any of the coordinators input 
/// // senders, any changes to the topology will result in a runtime panic.
/// tx.send(10);
/// ```
///
pub struct Coordinator<'a> {
    channels: Mutex<Vec<&'a NoOp>>,


    no_ops: Vec<Box<NoOp>>,
    no_op_txs: Vec<Box<NoOpTx>>,
}

impl Coordinator {
    pub fn new() -> Coordinator {
        let (tx, rx) = channel();
        Coordinator {
            exec_tx: tx,
            exec_rx: rx,
            no_ops: Vec::new(),
            no_op_txs: Vec::new()
        }
    }

    pub fn channel<A>(&self) -> (Sender<A>, Signal<A>) 
    where A: 'static + Send + Clone,
    {
        // For data coming in
        let (data_tx, data_rx): (Sender<A>, Receiver<A>) = channel();

        // For data going out, type Option<A>
        let (signal_tx, signal_rx): (Sender<Option<A>>, Receiver<Option<A>>) = channel();
        let signal = Signal::new(&self, signal_rx);

        // Multiplexing channel for both data and new channels
        // type Either<A>
        let (either_tx, either_rx): (Sender<Either<A>>, Receiver<Either<A>>) = channel();

        /*
        // Notify existing channels of me
        for no_op_tx in self.no_op_txs.iter() {
            no_op_tx.send(Box::new(signal_tx.clone()));
        }

        // Ensure future channels push no-ops to me
        self.no_ops.push(Box::new(signal_tx.clone()));

        // Process multiplexed events
        let mut no_ops: Vec<Box<NoOp>> = self.no_ops.iter().map(|i| i.boxed_clone()).collect();
        thread::spawn(move || {
            let out = signal_tx;
            let mut fuse = false;

            loop {
                match either_rx.recv() {
                    Ok(Either::Data(x)) => {
                        fuse = true;
                        out.send(Some(x));
                        for no_op in no_ops.iter() {
                            no_op.no_op();
                        }
                    }
                    Ok(Either::Channel(no_op)) => {
                        if fuse {
                            panic!("Cannot add channels to a Coordinator that has begun data processing");
                        }
                        no_ops.push(no_op);
                    }
                    _ => { return; }
                }
            }
        });

        // Allow future channels to notify me of them so I can push no-ops to them
        self.no_op_txs.push(Box::new(either_tx.clone()));
        */

        return (data_tx, signal);
    }
}
