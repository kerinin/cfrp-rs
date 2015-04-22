use std::sync::*;
use std::sync::mpsc::*;

use super::{Fork, Branch, Signal, InternalSignal, Run, Push, Event, Lift, Fold};

// Fork is the incoming portion of a fork.  It is pushed data from upstream and
// clones it across a (possibly empty) set of child branches
//
impl<A> Run for Fork<A> where
    A: 'static + Clone + Send,
{
    fn run(self: Box<Self>) {
        let inner = *self;
        let Fork { parent, sink_txs } = inner;

        parent.push_to(
            Box::new(
                ForkPusher {
                    sink_txs: sink_txs,
                }
            )
        )
    }
}

struct ForkPusher<A> {
    sink_txs: Arc<Mutex<Vec<Sender<Event<A>>>>>,
}

impl<A> Push<A> for ForkPusher<A> where
    A: 'static + Clone + Send,
{
    fn push(&mut self, event: Event<A>) {
        for sink_tx in self.sink_txs.lock().unwrap().iter() {
            match sink_tx.send(event.clone()) {
                // We can't really terminate a child process, so just ignore errors...
                _ => {},
            }
        }
    }
}

// Branch is the outgoig portion of a fork.  It spawns waits for incoming data 
// from it's parent fork and pushes it to its children
//
impl<A> InternalSignal<A> for Branch<A> where
    A: 'static + Send,
{
    fn push_to(self: Box<Self>, mut target: Box<Push<A>>) {
        loop {
            match self.source_rx.recv() {
                Ok(event) => target.push(event),
                Err(_) => return,
            }
        }
    }
}

impl<A> Clone for Branch<A> where
    A: 'static + Send,
{
    fn clone(&self) -> Branch<A> {
        let (tx, rx) = channel();
        self.fork_txs.lock().unwrap().push(tx);
        Branch { fork_txs: self.fork_txs.clone(), source_rx: rx }
    }
}

impl<A> Branch<A>
{
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
    pub fn lift<F, B>(self, f: F) -> Signal<B> where
        F: 'static + Send + Fn(A) -> B,
        A: 'static + Send,
        B: 'static + Send,
    {
        Signal {
            internal_signal: Box::new(Lift::new(Box::new(self), f)),
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
    pub fn foldp<F, B>(self, initial: B, f: F) -> Signal<B> where
        F: 'static + Send + FnMut(&mut B, A),
        A: 'static + Send,
        B: 'static + Send + Clone,
    {
        Signal {
            internal_signal: Box::new(Fold::new(Box::new(self), f, initial))
        }
    }
}
