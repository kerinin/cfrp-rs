use std::sync::*;
use std::sync::mpsc::*;

use super::*;
use super::super::Signal;
use super::lift::Lift;
use super::fold::Fold;

// A Fork is created internally when Builder#add is called.  The purpose of Fork is
// to distribute incoming data to some number of child Branch instances.
//
// NOTE: We spin up a child on add and probably send data to it - are we goig to
// get memory leaks?
//
// Fork is the "root" of the topology; when a topology is started, `run` is 
// called for each Fork in the topology, which causes `push_to` to be called
// for all the nodes upstream of the Fork. Forks run in their data source's 
// thread.
//
pub struct Fork<A> where
    A: 'static + Send,
{
    parent: Box<InternalSignal<A>>,
    sink_txs: Arc<Mutex<Vec<Sender<Event<A>>>>>,
}

impl<A> Fork<A> where
    A: 'static + Clone + Send,
{
    pub fn new(parent: Box<InternalSignal<A>>, sink_txs: Arc<Mutex<Vec<Sender<Event<A>>>>>) -> Fork<A> {
        Fork {
            parent: parent,
            sink_txs: sink_txs,
        }
    }
}

// Fork is the incoming portion of a fork.  It is pushed data from upstream and
// clones it across a (possibly empty) set of child branches
//
impl<A> Run for Fork<A> where
    A: 'static + Clone + Send,
{
    fn run(self: Box<Self>) {
        let has_branches = !self.sink_txs.lock().unwrap().is_empty();

        if has_branches {
            println!("Fork::run with branches");

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
            println!("Fork::run without branches");

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
        println!("ForkPusher handling Event");

        for sink_tx in self.sink_txs.lock().unwrap().iter() {
            match sink_tx.send(event.clone()) {
                // We can't really terminate a child process, so just ignore errors...
                _ => {},
            }
        }
    }
}

/// A data source of type `A` which can be used as input more than once
///
/// This operation is equivalent to a "let" binding, or variable assignment.
/// Branch implements `Clone`, and each clone runs in its own thread.
///
/// Branches are returned when `add` is called on a `Builder`
///
pub struct Branch<A> where
    A: 'static + Send,
{
    fork_txs: Arc<Mutex<Vec<Sender<Event<A>>>>>,
    source_rx: Option<Receiver<Event<A>>>,
}

impl<A> Branch<A> where
    A: 'static + Send,
{
    pub fn new(fork_txs: Arc<Mutex<Vec<Sender<Event<A>>>>>, source_rx: Option<Receiver<Event<A>>>) -> Branch<A> {
        Branch {
            fork_txs: fork_txs,
            source_rx: source_rx,
        }
    }
}

// Branch is the outgoig portion of a fork.  It waits for incoming data 
// from it's parent fork and pushes it to its children
//
impl<A> InternalSignal<A> for Branch<A> where
    A: 'static + Send,
{
    fn push_to(self: Box<Self>, target: Option<Box<Push<A>>>) {
        match (target, self.source_rx) {
            (Some(mut t), Some(rx)) => {
                println!("Branch::push_to with target");

                loop {
                    match rx.recv() {
                        Ok(event) => t.push(event),
                        Err(_) => return,
                    }
                }
            },
            (None, Some(rx)) => {
                println!("Branch::push_to with empty target");

                // Just ensuring the channel is drained so we don't get memory leaks
                loop {
                    match rx.recv() {
                        Err(_) => return,
                        _ => {},
                    }
                }
            }
            _ => {
                println!("Branch::push_to with neither target nor source")
            },
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
        A: 'static + Send + Clone,
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
