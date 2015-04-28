//! # Concurrent Function Reactive Programming
//!
//! Highly influenced by [Elm](http://elm-lang.org/) - provides a framework for describing &
//! executing 
//! concurrent data flow processes.
//!
//! # Examples
//!
//! ```
//! use std::sync::mpsc::*;
//! use cfrp::*;
//!
//! // create some channels for communicating with the topology
//! let (in_tx, in_rx) = channel();
//! //let (out_tx, out_rx) = channel();
//! 
//! // Topologies are statically defined, run-once structures.  Due to how
//! // concurrency is handled, changes to the graph structure can cause
//! // inconsistencies in the data processing
//! // 
//! // You can pass some state in (here we're passing `(in_rx, out_rx)`) if you need
//! // to.
//! spawn_topology( in_rx, |t, in_rx| {
//! 
//!     // Create a listener on `in_rx`.  Messages received on the channel will be
//!     // sent to any nodes subscribed to `input`
//!     let input = t.add(t.listen(0, in_rx));
//! 
//!     // Basic map operation.  Since this is a pure function, it will only be
//!     // evaluated when the value of `input` changes
//!     let plus_one = t.add(input.lift(|i| -> usize { i + 1 }));
//! 
//!     // The return value of `add` implements `Clone`, and can be used to
//!     // 'fan-out' data
//!     let plus_two = plus_one.clone().lift(|i| -> usize { i + 2 });
//! 
//!     // We can combine signals too.  Since it's possible to receive input on one
//!     // side but not the other, `lift2` always passes `Option<T>` to its
//!     // function.  Like `lift`, this function is only called when needed
//!     let combined = plus_one.lift2(plus_two, |i, j| -> usize {
//!         println!("lifting");
//!         match (i, j) {
//!             (Some(a), Some(b)) => a + b,
//!             _ => 0,
//!         } 
//!     });
//! 
//!     // `fold` allows us to track state across events.  Since this is assumed to
//!     // be impure, it is called any time a signal is received upstream of
//!     // `combined`.
//!     let accumulated = combined.fold(0, |sum, i| { *sum += i; });
//! 
//!     // Make sure to add transformations to the topology - if it's not added it
//!     // won't be run...
//!     t.add(accumulated);
//! });
//!
//! in_tx.send(1usize).unwrap();
//! // let out: usize = out_rx.recv().unwrap();
//! // assert_eq!(out, 2);
//! ```
//!
#[macro_use]
extern crate log;

pub mod primitives;
mod signal_ext;
mod topology;
mod builder;

pub use signal_ext::SignalExt;
pub use topology::{Topology, TopologyHandle};
pub use builder::Builder;


/// Container for data as it flows across the topology
#[derive(Clone)]
pub enum Event<A> {
    Changed(A),
    Unchanged,
    Exit,
}

#[derive(Clone)]
pub enum SignalType<A> {
    Constant(A),
    Dynamic(A),
}

impl<A> SignalType<A> {
    fn unwrap(self) -> A {
        match self {
            SignalType::Constant(v) => v,
            SignalType::Dynamic(v) => v,
        }
    }
}

/// Types which can serve as a data source
///
pub trait Signal<A>: Send where
A: 'static + Send + Clone,
{
    // Called at build time when a downstream process is created for the signal
    fn init(&mut self) {}

    // Returns the "initial" value of the signal
    fn initial(&self) -> SignalType<A>;

    // Called at compile time when a donstream process is run
    fn push_to(self: Box<Self>, Option<Box<Push<A>>>);
}

/// Types which can receive incoming data from other signals
///
pub trait Push<A> {
    fn push(&mut self, Event<A>);
}

pub trait Run: Send {
    fn run(mut self: Box<Self>);
}

/// Construct a new topology and run it
///
/// `f` will be called with a `Builder`, which exposes methods for adding
/// inputs & transformations to the topology and `state` which is provided as 
/// a convenience for passing values through to builder's scope.
///
/// # Example
///
/// ```
/// use cfrp::*;
///
/// let handle = spawn_topology((0, 1), |t, (init, incr)| {
///     t.add(
///         t.value(incr).fold(init, |sum, i| { *sum += i })
///     );
/// });
/// ```
///
pub fn spawn_topology<F>(f: F) -> TopologyHandle where
    F: FnOnce(&Builder),
{
    let builder = Builder::new();
    f(&builder);
    Topology::new(builder.inputs.into_inner(), builder.runners.into_inner()).run()
}

#[cfg(test)] 
mod test {
    extern crate env_logger;

    use std::sync::mpsc::*;
    use std::thread;

    use super::*;

    #[test]
    fn lift_channel() {
        let (in_tx, in_rx) = sync_channel(0);
        let (out_tx, out_rx) = channel();

        spawn_topology(move |t| {
            t.listen(0, in_rx)
                .lift(move |i| { out_tx.send(i | (1 << 1)).unwrap(); })
                .add_to(t);
        });

        // Initial value
        assert_eq!(out_rx.recv().unwrap(), 0b00000010);

        // Lifted value
        in_tx.send(1).unwrap();
        assert_eq!(out_rx.recv().unwrap(), 0b00000011);
    }

    #[test]
    fn lift_value() {
        let (out_tx, out_rx) = channel();

        spawn_topology(move |t| {
            t.value(0)
                .lift(move |i| { out_tx.send(i | (1 << 1)).unwrap(); })
                .add_to(t);
        });

        // Initial value
        assert_eq!(out_rx.recv().unwrap(), 0b00000010);
    }

    #[test]
    fn fold_channel() {
        let (in_tx, in_rx) = sync_channel(0);
        let (out_tx, out_rx) = channel();

        spawn_topology(move |t| {
            t.listen(0, in_rx)
                .fold(out_tx, |tx, i| { tx.send(i | (1 << 1)).unwrap(); })
                .add_to(t);
        });

        // Initial value
        assert_eq!(out_rx.recv().unwrap(), 0b00000010);

        // Lifted value
        in_tx.send(1).unwrap();
        assert_eq!(out_rx.recv().unwrap(), 0b00000011);
    }

    #[test]
    fn fold_value() {
        let (out_tx, out_rx) = channel();

        spawn_topology(move |t| {
            t.value(0)
                .fold(out_tx, |tx, i| { tx.send(i | (1 << 1)).unwrap(); })
                .add_to(t);
        });

        // Initial value
        assert_eq!(out_rx.recv().unwrap(), 0b00000010);
    }

    #[test]
    fn lift2_channel_channel() {
        let (l_tx, l_rx) = sync_channel(0);
        let (r_tx, r_rx) = sync_channel(0);
        let (out_tx, out_rx) = channel();

        spawn_topology(move |t| {
            t.listen(1 << 0, l_rx)
                .lift2(t.listen(1 << 1, r_rx), move |i,j| { out_tx.send(i | j).unwrap() })
                .add_to(t);
        });

        // Initial value
        assert_eq!(out_rx.recv().unwrap(), (1 << 0) | (1 << 1));

        l_tx.send(1 << 2).unwrap();
        assert_eq!(out_rx.recv().unwrap(), (1 << 2) | (1 << 1));

        r_tx.send(1 << 3).unwrap();
        assert_eq!(out_rx.recv().unwrap(), (1 << 2) | (1 << 3));
    }

    #[test]
    fn lift2_channel_value() {
        let (tx, rx) = sync_channel(0);
        let (out_tx, out_rx) = channel();

        spawn_topology(move |t| {
            t.listen(1 << 0, rx)
                .lift2(t.value(1 << 1), move |i,j| { out_tx.send(i | j).unwrap() })
                .add_to(t);
        });

        // Initial value
        assert_eq!(out_rx.recv().unwrap(), (1 << 0) | (1 << 1));

        tx.send(1 << 2).unwrap();
        assert_eq!(out_rx.recv().unwrap(), (1 << 2) | (1 << 1));
    }

    #[test]
    fn lift2_value_value() {
        let (out_tx, out_rx) = channel();

        spawn_topology(move |t| {
            t.value(1 << 0)
                .lift2(t.value(1 << 1), move |i,j| { out_tx.send(i | j).unwrap() })
                .add_to(t);
        });

        // Initial value
        assert_eq!(out_rx.recv().unwrap(), (1 << 0) | (1 << 1));
    }

    #[test]
    fn async_sends() {
        let (tx, rx) = sync_channel(0);
        let (out_tx, out_rx) = channel();

        spawn_topology(move |t| {
            t.async(
                t.listen(1 << 0, rx)
                .lift(move |i| { out_tx.send(i).unwrap(); })
            );
        });

        assert_eq!(out_rx.recv().unwrap(), (1 << 0));

        tx.send(1 << 1).unwrap();
        assert_eq!(out_rx.recv().unwrap(), (1 << 1)); // Should receive fast output first
    }

    #[test]
    fn async_sends_async() {
        env_logger::init().unwrap();

        let (slow_tx, slow_rx) = channel();
        let (fast_tx, fast_rx) = channel();
        let (out_tx, out_rx) = channel();

        spawn_topology(move |t| {
            let slow = t.listen(1 << 0, slow_rx)
                .lift(|i| -> usize { 
                    if i > 1 { // allow the initial value to be computed quickly
                        thread::sleep_ms(100);
                    }

                    i 
                }).async(t);

            let fast = t.listen(1 << 1, fast_rx);

            slow.lift2(fast, move |i,j| { out_tx.send(i | j).unwrap() })
            .add_to(t);
        });

        // Initial value
        assert_eq!(out_rx.recv().unwrap(), (1 << 0) | (1 << 1));

        slow_tx.send(1 << 2).unwrap();
        fast_tx.send(1 << 3).unwrap();

        // Should receive the 'fast' value first...
        assert_eq!(out_rx.recv().unwrap(), (1 << 0) | (1 << 3));
        // ...then the slow one
        assert_eq!(out_rx.recv().unwrap(), (1 << 2) | (1 << 3));
    }

    #[test]
    fn branch() {
        let (tx, rx) = channel();
        let (out_tx1, out_rx1) = channel();
        let (out_tx2, out_rx2) = channel();

        spawn_topology(move |t| {
            let a = t.listen(1 << 0, rx);

            t.add(a.clone().lift(move |i| { out_tx1.send(i).unwrap(); }));
            t.add(a.clone().lift(move |i| { out_tx2.send(i).unwrap(); }));
        });

        assert_eq!(out_rx1.recv().unwrap(), (1 << 0));
        assert_eq!(out_rx2.recv().unwrap(), (1 << 0));

        tx.send(1 << 1).unwrap();
        assert_eq!(out_rx1.recv().unwrap(), (1 << 1));
        assert_eq!(out_rx2.recv().unwrap(), (1 << 1));
    }
}
