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
// mod topology;

// pub use topology::{Topology, Builder};
use primitives::{Topology, Builder, TopologyHandle};
use primitives::LiftSignal;
use primitives::Lift2Signal;
use primitives::FoldSignal;
use primitives::Value;




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

/// Types which can serve as a data source
///
pub trait Signal<A>: Send {
    // Called at build time when a downstream process is created for the signal
    fn init(&mut self) {}

    // Returns the "initial" value of the signal
    fn initial(&self) -> SignalType<A>;

    // Called at compile time when a donstream process is run
    fn push_to(self: Box<Self>, Option<Box<Push<A>>>);
}

pub trait Let<A>: Signal<A> {
    fn clone(self: Box<Self>) -> Box<Signal<A>>; 
}

/// Types which can receive incoming data from other signals
///
pub trait Push<A> {
    fn push(&mut self, Event<A>);
}

/// Apply a pure function `F` to a data source `Signal<A>`, generating a 
/// transformed output data source `Signal<B>`.
///
/// Other names for this operation include "map" or "collect".  `f` will be
/// run in `self`'s thread
///
/// Because `F` is assumed to be pure, it will only be evaluated for
/// new data that has changed since the last observation.  If side-effects are
/// desired, use `fold` instead.
///
/// # Example
///
/// ```
/// use cfrp::*;
/// use cfrp::primitives::*;
///
/// // Create a constant-valued channel with value 1 and return a channel with
/// // value 2.  The value will only be calculated with `i` changes (which in 
/// // this case is never)
/// Builder::new().value(1)
///     .lift(|i| { i + 1 });
/// ```
///
pub trait Lift<A>: Signal<A> + Sized {
    fn lift<F, B>(mut self, f: F) -> Box<Signal<B>> where
        Self: 'static,
        F: 'static + Send + Fn(A) -> B,
        A: 'static + Send,
        B: 'static + Send + Clone,
    {
        self.init();

        match self.initial() {
            SignalType::Dynamic(v) => {
                let initial = f(v);
                Box::new(LiftSignal::new(Box::new(self), f, initial))
            }
            SignalType::Constant(v) => {
                let initial = f(v);
                Box::new(Value::new(initial))
            }
        }
    }
}

/// Apply a pure function `F` to a two data sources `Signal<A>`, and `Signal<B>`,
/// generating a transformed output data source `Signal<C>`.
///
/// Other names for this operation include "map" or "collect".  `f` will be
/// run in `self`'s thread
///
/// Because `F` is assumed to be pure, it will only be evaluated for
/// new data that has changed since the last observation.  If side-effects are
/// desired, use `fold` instead.
///
/// # Example
///
/// ```
/// use cfrp::*;
/// use cfrp::primitives::*;
///
/// let b = Builder::new();
///
/// // Create two constant-valued channels with value 1 and return a channel with
/// // their sum.  The value will only be calculated with `i` or `j` changes 
/// // (which in this case is never)
/// b.value(1).lift2(b.value(1), |i, j| { 
///
///     // All data entering the topology is processed by every node in the 
///     // topology, this means that data entering a given channel will be `None`
///     // downstream of other channels.
///     match (i, j) {
///         (Some(i), Some(j))  => { i + j },
///         (Some(i), None)     => i,
///         (None, Some(j))     => j,
///         (None, None)        => 0,
///     }
/// });
/// ```
///
pub trait Lift2<A, B, SB>: Signal<A> + Sized {
    fn lift2<F, C>(mut self, mut right: SB, f: F) -> Box<Signal<C>> where
        Self: 'static,
        SB: 'static + Signal<B>,
        F: 'static + Send + Fn(A, B) -> C,
        A: 'static + Send + Clone,
        B: 'static + Send + Clone,
        C: 'static + Send + Clone,
    {
        self.init();
        right.init();

        match (self.initial(), right.initial()) {
            (SignalType::Dynamic(l), SignalType::Dynamic(r)) => {
                let initial = f(l,r);
                Box::new(Lift2Signal::new(Box::new(self), Box::new(right), f, initial))
            }

            (SignalType::Dynamic(l), SignalType::Constant(r)) => {
                let initial = f(l, r.clone());
                let signal = LiftSignal::new(
                    Box::new(self),
                    move |dynamic_l| -> C { f(dynamic_l, r.clone()) },
                    initial,
                );

                Box::new(signal)
            }

            (SignalType::Constant(l), SignalType::Dynamic(r)) => {
                let initial = f(l.clone(), r);
                let signal = LiftSignal::new(
                    Box::new(right),
                    move |dynamic_r| -> C { f(l.clone(), dynamic_r) },
                    initial,
                );

                Box::new(signal)
            }

            (SignalType::Constant(l), SignalType::Constant(r)) => {
                let initial = f(l, r);
                Box::new(Value::new(initial))
            }
        }
    }
}

/// Apply a function `F` which uses a data source `Signal<A>` to 
/// mutate an instance of `B`, generating an output data source `Signal<B>`
/// containing the mutated value
///
/// Other names for this operation include "reduce" or "inject".  `f` will
/// be run in `self`'s thread
///
/// Fold is assumed to be impure, therefore the function will be called with
/// all data upstream of the fold, even if there are no changes in the stream.
///
/// # Example
///
/// ```
/// use cfrp::*;
/// use cfrp::primitives::*;
///
/// // Create a constant-valued channel with value 1 and fold it into a summation
/// Builder::new().value(1)
///     .fold(0, |sum, i| { *sum += i });
/// ```
///
pub trait Fold<A>: Signal<A> + Sized {
    fn fold<F, B>(mut self, mut initial: B, mut f: F) -> Box<Signal<B>> where
        Self: 'static,
        F: 'static + Send + FnMut(&mut B, A),
        A: 'static + Send + Clone,
        B: 'static + Send + Clone,
    {
        self.init();

        match self.initial() {
            SignalType::Dynamic(v) => {
                f(&mut initial, v);
                Box::new(FoldSignal::new(Box::new(self), initial, f))
            }

            SignalType::Constant(v) => {
                f(&mut initial, v);
                Box::new(Value::new(initial))
            }
        }
    }
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
pub fn spawn_topology<T, F>(state: T, f: F) -> TopologyHandle where
    F: Fn(&Builder, T),
{
    let builder = Builder::new();
    f(&builder, state);
    Topology::new(builder).run()
}

#[cfg(test)] 
mod test {
    extern crate env_logger;

    use std::sync::mpsc::*;

    use super::*;

    #[test]
    fn integration() {
        env_logger::init().unwrap();

        let (out_tx, out_rx) = sync_channel(0);

        spawn_topology(out_tx, |t, out_tx| {

            let input = t.add(t.value(1));

            t.add(input.clone()
                  .lift(|i| -> usize { i })
                  .lift2(input, |i, j| -> usize { i + j })
                  .fold(out_tx, |tx, a| {
                      match tx.send(a) {
                          Err(e) => { panic!("Error sending {}", e); },
                          _ => {},
                      };
                  })
                 );
        });

        let out = match out_rx.recv() {
            Err(e) => panic!("Failed to receive: {}", e),
            Ok(v) => v,
        };

        assert_eq!(out, 2);
    }
}
