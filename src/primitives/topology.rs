use std::cell::*;
use std::sync::*;
use std::sync::mpsc::*;
use std::thread;
use std::marker::*;

use super::super::{Signal, SignalType, Let};
use super::input::{RunInput, NoOp, ReceiverInput};
use super::fork::{Run, Fork, Branch};
use super::channel::Channel;
use super::async::Async;
use super::value::Value;

/// `Builder` is used to construct topologies.  
///
/// Basic builder pattern - `Topology::build` accepts a function which takes
/// a state type `T` and a mutable builder.  The builder can be used to create
/// `Channel`s and to `add` nodes to the topology
///
pub struct Builder {
    inputs: RefCell<Vec<Box<RunInput>>>,
    root_signals: RefCell<Vec<Box<Run>>>,
}

impl Builder {
    /// Create a new Builder
    ///
    pub fn new() -> Self {
        Builder {
            root_signals: RefCell::new(Vec::new()),
            inputs: RefCell::new(Vec::new()),
        }
    }

    /// Add a signal to the topology
    ///
    /// Returns a `Branch<A>`, allowing `root` to be used as input more than once
    ///
    /// # Example
    ///
    /// ```
    /// use cfrp::*;
    /// use cfrp::primitives::*;
    ///
    /// let b = Builder::new();
    /// 
    /// // Topologies only execute transformations which have been added to a builder.
    /// let fork = b.add(b.value(1).lift(|i| { i + 1} ));
    ///
    /// // `add` returns a signal that can be used more than once
    /// b.add(fork.clone().lift(|i| { i - 1 } ));
    /// b.add(fork.lift(|i| { -i }));
    /// ```
    ///
    pub fn add<A>(&self, root: Box<Signal<A>>) -> Box<Signal<A>> where // NOTE: This needs to be clone-able!
        A: 'static + Clone + Send,
    {
        match root.initial() {
            SignalType::Dynamic(v) => {
                let fork_txs = Arc::new(Mutex::new(Vec::new()));

                let fork = Fork::new(root, fork_txs.clone());

                self.root_signals.borrow_mut().push(Box::new(fork));

                Box::new(Branch::new(fork_txs, None, v))
            }

            SignalType::Constant(v) => {
                Box::new(Value::new(v))
            }
        }
    }

    /// Listen to `input` and push received data into the topology
    ///
    /// All data must enter the topology via a call to `listen`; this function
    /// ensures data syncronization across the topology.  Each listener runs in 
    /// its own thread
    ///
    /// # Example
    ///
    /// ```
    /// use std::sync::mpsc::*;
    /// use cfrp::*;
    /// use cfrp::primitives::*;
    ///
    /// let b = Builder::new();
    /// 
    /// let (tx, rx): (Sender<usize>, Receiver<usize>) = channel();
    ///
    /// // Receive data on `rx` and expose it as a signal with initial value 
    /// //`initial`.  This is necessary because the topology must maintain 
    /// // consistency between threads, so any message sent to any input is 
    /// // propagated to all other inputs as "no-change" messages.
    /// let signal = b.listen(0, rx);
    /// ```
    ///
    pub fn listen<A>(&self, initial: A, input: Receiver<A>) -> Box<Signal<A>> where
        A: 'static + Clone + Send,
    {
        let (tx, rx) = sync_channel(0);

        let runner = ReceiverInput::new(input, tx);

        self.inputs.borrow_mut().push(Box::new(runner));

        Box::new(Channel::new(rx, initial))
    }

    /// Combination of adding a signal and a channel
    ///
    /// Async allows signals to be processed downstream out of order.  Internally,
    /// the output of `root` is sent to new input channel.  The result is that
    /// long-running processes can be handled outside of the synchronized topology
    /// process, and the result can be handled when it's available.
    ///
    /// ```
    /// use std::sync::mpsc::*;
    /// use cfrp::*;
    /// use cfrp::primitives::*;
    ///
    /// let b = Builder::new();
    /// 
    /// // This will now happen without blocking the rest of the topology
    /// let result = b.async(
    ///     b.value(0).fold(0, |i, j| {
    ///         // Some very expensive code in here...
    ///     })
    /// );
    ///
    /// // ...and `result` will receive the output value when it's done
    /// b.add(
    ///     b.value(0).lift2(result, |i, j| { (i, j) })
    /// );
    /// ```
    ///
    pub fn async<A>(&self, root: Box<Signal<A>>) -> Box<Signal<A>> where // NOTE: Needs to be cloneable
        A: 'static + Clone + Send,
    {

        match root.initial() {
            SignalType::Dynamic(v) => {
                let (tx, rx) = sync_channel(0);
                let pusher = Async::new(root, tx);
                self.root_signals.borrow_mut().push(Box::new(pusher));

                self.listen(v, rx)
            }

            SignalType::Constant(v) => {
                Box::new(Value::new(v))
            }
        }
    }

    /// Creats a channel with constant value `v`
    ///
    pub fn value<T>(&self, v: T) -> Box<Signal<T>> where
        T: 'static + Clone + Send,
    {
        Box::new(Value::new(v))
    }
}

/// `Topology<T>` describes a data flow and controls its execution
///
/// If a record of type `T` is passed to `build`, it will be proxied into the
/// builder function as the second argument.  This allows data to be passed from
/// outside the builder's scope into the topology.
///
pub struct Topology {
    builder: Builder,
}

impl Topology {
    /// Create a new topology from a builder
    ///
    pub fn new(builder: Builder) -> Self {
        Topology { builder: builder }
    }

    /// Run the topology
    ///
    // pub fn run(self) -> TopologyHandle {
    pub fn run(self) {
        info!("----> TOPOLOGY STARTING");
        let Builder {inputs, root_signals} = self.builder;

        for root_signal in root_signals.into_inner().into_iter() {
            thread::spawn(move || {
                root_signal.run();
            });
        }

        let no_ops = Arc::new(Mutex::new(inputs.borrow().iter().map(|i| i.boxed_no_op()).collect::<Vec<Box<NoOp>>>()));
        // let term_txs = inputs.borrow().iter().map(|i| i.boxed_no_op()).collect::<Vec<Box<NoOp>>>();
        for (idx, input) in inputs.into_inner().into_iter().enumerate() {
            let no_ops_i = no_ops.clone();
            thread::spawn(move || {
                input.run(idx, no_ops_i);
            });
        }

        info!("----> TOPOLOGY RUNNING...");
        //
        // TopologyHandle {
        //     term_txs: term_txs,
        // }
    }
}

/// For explicitly terminating a running topology
///
pub struct TopologyHandle {
    term_txs: Vec<Box<NoOp>>,
}

impl Drop for TopologyHandle {
    fn drop(&mut self) {
        for tx in self.term_txs.iter() {
            tx.send_exit();
        }
        debug!("----> TOPOLOGY DROPPED");
    }
}
