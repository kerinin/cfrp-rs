use std::thread;
use std::iter;
use std::cell::*;
use std::sync::*;
use std::sync::mpsc::*;
use std::marker::*;
use std::ops::Add;

use rand;
use time;

use super::{Signal, SignalExt, Run, Config};
use primitives::input::{RunInput, ReceiverInput, AckInput, RngInput};
use primitives::fork::{Fork, Branch};
use primitives::channel::Channel;
use primitives::async::Async;
use primitives::value::Value;

/// `Builder` provides helpers for building topologies
///
pub struct Builder {
    config: Config,
    pub inputs: RefCell<Vec<Box<RunInput>>>,
    pub runners: RefCell<Vec<Box<Run>>>,
}

impl Builder {
    /// Create a new Builder
    ///
    pub fn new(config: Config) -> Self {
        Builder {
            config: config,
            runners: RefCell::new(Vec::new()),
            inputs: RefCell::new(Vec::new()),
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
    /// use std::default::*;
    /// use std::sync::mpsc::*;
    /// use cfrp::*;
    ///
    /// let b = Builder::new(Default::default());
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
    pub fn listen<A>(&self, initial: A, input: Receiver<A>) -> Branch<A> where
        A: 'static + Clone + Send,
    {
        let (tx, rx) = sync_channel(self.config.buffer_size.clone());

        let runner = ReceiverInput::new(input, tx);

        self.inputs.borrow_mut().push(Box::new(runner));

        self.add(Channel::new(self.config.clone(), rx, initial))
    }

    /// Creats a channel with constant value `v`
    /// 
    /// Nodes downstream of values will be executed once on initialization and
    /// never again.  Constant values can be combined with dynamic values via
    /// `lift2`, in which case nodes will be executed when their dynamic parents
    /// change values.
    ///
    /// # Example
    ///
    /// ```
    /// use std::default::*;
    /// use std::sync::mpsc::*;
    /// use cfrp::*;
    ///
    /// let b = Builder::new(Default::default());
    /// let (tx, rx): (Sender<usize>, Receiver<usize>) = channel();
    ///
    /// let v = b.value(0);
    /// let ch = b.listen(0, rx);
    ///
    /// // Only ever computed once
    /// let l1 = v.lift(|i| { i + 1 });
    ///
    /// // Computed any time `ch` receives data, `static` will always be `0`
    /// let l2 = l1.lift2(ch, |st, dy| { *st + *dy });
    /// ```
    ///
    pub fn value<T>(&self, v: T) -> Value<T> where
        T: 'static + Clone + Send,
    {
        Value::new(self.config.clone(), v)
    }

    /// Returns a signal which emits the "current" time every at every `interval`
    ///
    /// The actual time between events may be longer than `interval` if
    /// writing to the topology blocks or the scheduling thread is pre-empted
    /// by other threads, however a signal with every interval's time value will
    /// eventually be sent.
    /// 
    pub fn every(&self, interval: time::Duration) -> Branch<time::Tm>
    {
        let (tx, rx) = sync_channel(0);
        let initial = time::now();

        let mut last_tick = initial.clone();
        thread::spawn(move || {
            loop {
                let tm = iter::repeat(interval)
                    .scan(last_tick.clone(), |tm, d| {
                        *tm = *tm + d;
                        Some(*tm)
                    })
                    .inspect(|tm| { 
                        match tx.send(tm.clone()) {
                            Ok(_) => { last_tick = *tm; },
                            Err(_) => return,
                        }
                    })
                    .take_while(|tm| { *tm < time::now() })
                    .last();

                let sleep_duration = time::now() - tm.unwrap();
                thread::sleep_ms(sleep_duration.num_milliseconds() as u32);
            }
        });

        self.listen(initial, rx)
    }

    /// Creates a channel which pushes `Event::Changed(initial)` when any 
    /// other channel receives changes
    ///
    /// Signals created with `listen` only cause nodes directly downstream of 
    /// themselves to be recomputed. By contrast, signals created by `ack_*` will
    /// emit a value when any input signal's value changes.  
    ///
    /// # Example
    /// ```
    /// use std::sync::mpsc::*;
    /// use cfrp::*;
    ///
    /// let(tx, rx) = channel();
    /// let(out_tx, out_rx) = channel();
    ///
    /// spawn_topology(Default::default(), move |t| {
    ///     t.add(t.ack_value(1).lift(move |i| { out_tx.send(i).unwrap(); }));
    ///     t.add(t.listen(0, rx));
    /// });
    ///
    /// // Initial
    /// assert_eq!(out_rx.recv().unwrap(), 1);
    ///
    /// tx.send(1).unwrap();
    /// assert_eq!(out_rx.recv().unwrap(), 1);
    /// ```
    ///
    pub fn ack_value<A>(&self, initial: A) -> Branch<A> where
        A: 'static + Clone + Send,
    {
        let (tx, rx) = sync_channel(self.config.buffer_size.clone());

        let runner = AckInput::new(initial.clone(), tx);

        self.inputs.borrow_mut().push(Box::new(runner));

        self.add(Channel::new(self.config.clone(), rx, initial))
    }

    /// Return a signal that increments each time the topology receives data
    ///
    /// Signals created with `listen` only cause nodes directly downstream of 
    /// themselves to be recomputed. By contrast, signals created by `ack_*` will
    /// emit a value when any input signal's value changes.  
    ///
    pub fn ack_counter<A, B>(&self, initial: A, by: A) -> Branch<A> where
    A: 'static + Clone + Send + Add<Output=A>,
    {
        self.add(
            self.ack_value(by)
            .fold(initial, |c, incr| { c + incr })
        )
    }

    /// Return a signal with the 'current' time each time the topology receives
    /// data
    ///
    /// Signals created with `listen` only cause nodes directly downstream of 
    /// themselves to be recomputed. By contrast, signals created by `ack_*` will
    /// emit a value when any input signal's value changes.  
    ///
    pub fn ack_timestamp(&self) -> Branch<time::Tm>
    {
        self.add(
            self.ack_value(())
            .lift(|_| -> time::Tm { time::now() })
        )
    }

    /// Return a signal which generates a random value each time the topology
    /// receives data
    ///
    /// If randomness is needed, `ack_random` is probably a better way of generating it
    /// than creating a Rng inside a handler because it exposes the generated 
    /// value as a 'fact' about the system's history rather than embedding it
    /// in a non-deterministic function.  
    ///
    /// Signals created with `listen` only cause nodes directly downstream of 
    /// themselves to be recomputed. By contrast, signals created by `ack_*` will
    /// emit a value when any input signal's value changes.  
    ///
    pub fn ack_random<R, A>(&self, mut rng: R) -> Branch<A> where
    R: 'static + rand::Rng + Clone + Send,
    A: 'static + Send + Clone + rand::Rand,
    {
        let (tx, rx) = sync_channel(self.config.buffer_size.clone());

        let initial = rng.gen();
        let runner = RngInput::new(rng, tx);

        self.inputs.borrow_mut().push(Box::new(runner));

        self.add(Channel::new(self.config.clone(), rx, initial))
    }

    /// Add a signal to the topology
    ///
    /// Returns a `Branch<A>`, allowing `root` to be used as input more than once
    /// `SignalExt<A>` also provides `add_to(&Builder)` so `Builder::add` can be
    /// used with method-chaining syntax
    ///
    /// # Example
    ///
    /// ```
    /// use std::default::*;
    /// use cfrp::*;
    ///
    /// let b = Builder::new(Default::default());
    /// 
    /// // Topologies only execute transformations which have been added to a builder.
    /// let fork = b.add(b.value(1).lift(|i| { i + 1} ));
    ///
    /// // `add` returns a signal that can be used more than once
    /// fork
    ///     .clone()
    ///     .lift(|i| { i - 1 } )
    ///     .add_to(&b);
    ///
    /// fork
    ///     .lift(|i| { -i })
    ///     .add_to(&b);
    /// ```
    ///
    pub fn add<SA, A>(&self, root: SA) -> Branch<A> where // NOTE: This needs to be clone-able!
        SA: 'static + Signal<A>,
        A: 'static + Clone + Send,
    {
        let v = root.initial();

        let fork_txs = Arc::new(Mutex::new(Vec::new()));

        let fork = Fork::new(Box::new(root), fork_txs.clone());

        self.runners.borrow_mut().push(Box::new(fork));

        Branch::new(self.config.clone(), fork_txs, None, v)
    }

    /// Combination of adding a signal and a channel
    ///
    /// Async allows signals to be processed downstream out of order.  Internally,
    /// the output of `root` is sent to new input channel.  The result is that
    /// long-running processes can be handled outside of the synchronized topology
    /// process, and the result can be handled when it's available.
    ///
    /// ```
    /// use std::thread;
    /// use std::sync::mpsc::*;
    /// use cfrp::*;
    ///
    /// let (slow_tx, slow_rx) = channel();
    /// let (fast_tx, fast_rx) = channel();
    /// let (out_tx, out_rx) = channel();
    ///
    /// spawn_topology(Default::default(), move |t| {
    ///     let slow = t.listen(1 << 0, slow_rx)
    ///         .lift(|i| -> usize { 
    ///             if i > 1 { // allow the initial value to be computed quickly
    ///                 thread::sleep_ms(100);
    ///             }
    ///
    ///             i 
    ///         }).async(t);
    ///
    ///     let fast = t.listen(1 << 1, fast_rx);
    ///
    ///     slow.lift2(fast, move |i,j| { out_tx.send(*i | *j).unwrap() })
    ///     .add_to(t);
    /// });
    ///
    /// // Initial value
    /// assert_eq!(out_rx.recv().unwrap(), (1 << 0) | (1 << 1));
    ///
    /// slow_tx.send(1 << 2).unwrap();
    /// fast_tx.send(1 << 3).unwrap();
    ///
    /// // Will receive the 'fast' value first...
    /// assert_eq!(out_rx.recv().unwrap(), (1 << 0) | (1 << 3));
    /// // ...then the slow one
    /// assert_eq!(out_rx.recv().unwrap(), (1 << 2) | (1 << 3));
    /// ```
    ///
    pub fn async<SA, A>(&self, root: SA) -> Branch<A> where // NOTE: Needs to be cloneable
        SA: 'static + Signal<A>,
        A: 'static + Clone + Send,
    {
        let v = root.initial();
        let (tx, rx) = sync_channel(self.config.buffer_size.clone());
        let pusher = Async::new(Box::new(root), tx);
        self.runners.borrow_mut().push(Box::new(pusher));

        self.listen(v.unwrap(), rx)
    }

}
