use std::sync::*;
use std::sync::mpsc::*;

use super::{Fork, Branch, Signal, InternalSignal, Run, Push, Event, Lift, Fold, LiftN, PullInputs, InputList};

// Fork is the incoming portion of a fork.  It is pushed data from upstream and
// clones it across a (possibly empty) set of child branches
//
impl<A> Run for Fork<A> where
    A: 'static + Clone + Send,
{
    fn run(self: Box<Self>) {
        let has_branches = !self.sink_txs.lock().unwrap().is_empty();

        if has_branches {
            let inner = *self;
            let Fork { parent, sink_txs } = inner;

            parent.push_to(
                Some(
                    Box::new(
                        ForkPusher {
                            sink_txs: sink_txs,
                        }
                    )
                )
            )
        } else {
            self.parent.push_to(None);
        }
                
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
    fn push_to(self: Box<Self>, target: Option<Box<Push<A>>>) {
        match (target, self.source_rx) {
            (Some(mut t), Some(rx)) => {
                loop {
                    match rx.recv() {
                        Ok(event) => t.push(event),
                        Err(_) => return,
                    }
                }
            },
            (None, Some(rx)) => {
                // Just ensuring the channel is drained so we don't get memory leaks
                loop {
                    match rx.recv() {
                        Err(_) => return,
                        _ => {},
                    }
                }
            }
            _ => {},
        }
    }
}

impl<A> Clone for Branch<A> where
    A: 'static + Send,
{
    fn clone(&self) -> Branch<A> {
        Branch { fork_txs: self.fork_txs.clone(), source_rx: None }
    }
}

impl<A> Branch<A> where
    A: 'static + Send,
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
    pub fn lift<F, B>(mut self, f: F) -> Signal<B> where
        F: 'static + Send + Fn(A) -> B,
        A: 'static + Send,
        B: 'static + Send,
    {
        let (tx, rx) = channel();
        self.fork_txs.lock().unwrap().push(tx);
        self.source_rx = Some(rx);

        Signal {
            internal_signal: Box::new(
                Lift {
                    parent: Box::new(self),
                    f: f,
                }
            )
        }
    }

    pub fn liftn<F, R, B>(mut self, rest: R, f: F) -> Signal<B> where
        F: 'static + Send + Fn(<<R as InputList<Box<InternalSignal<A>>>>::InputPullers as PullInputs>::Values) -> B,
        R: 'static + Send + InputList<Box<InternalSignal<A>>>,
        B: 'static + Send,
    {
        let (tx, rx) = channel();
        self.fork_txs.lock().unwrap().push(tx);
        self.source_rx = Some(rx);

        let head: Box<InternalSignal<A>> = Box::new(self);

        Signal {
            internal_signal: Box::new(
                LiftN {
                    head: head,
                    rest: rest,
                    f: f,
                }
            ),
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
    pub fn foldp<F, B>(mut self, initial: B, f: F) -> Signal<B> where
        F: 'static + Send + FnMut(&mut B, A),
        A: 'static + Send,
        B: 'static + Send + Clone,
    {
        let (tx, rx) = channel();
        self.fork_txs.lock().unwrap().push(tx);
        self.source_rx = Some(rx);

        Signal {
            internal_signal: Box::new(
                Fold {
                    parent: Box::new(self),
                    f: f,
                    state: initial,
                }
            )
        }
    }
}
