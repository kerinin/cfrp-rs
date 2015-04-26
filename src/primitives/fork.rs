use std::sync::*;
use std::sync::mpsc::*;

use super::super::{Event, Signal, SignalType, Push};

pub trait Run: Send {
    fn run(mut self: Box<Self>);
}

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
    parent: Box<Signal<A>>,
    sink_txs: Arc<Mutex<Vec<Sender<Event<A>>>>>,
}

impl<A> Fork<A> where
    A: 'static + Clone + Send,
{
    pub fn new(parent: Box<Signal<A>>, sink_txs: Arc<Mutex<Vec<Sender<Event<A>>>>>) -> Fork<A> {
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
            debug!("Fork::run with branches");

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
            debug!("Fork::run without branches");

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
        debug!("ForkPusher handling Event");

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
    initial: A,
}

impl<A> Branch<A> where
    A: 'static + Send,
{
    pub fn new(fork_txs: Arc<Mutex<Vec<Sender<Event<A>>>>>, source_rx: Option<Receiver<Event<A>>>, initial: A) -> Branch<A> {
        Branch {
            fork_txs: fork_txs,
            source_rx: source_rx,
            initial: initial,
        }
    }
}

// Branch is the outgoig portion of a fork.  It waits for incoming data 
// from it's parent fork and pushes it to its children
//
impl<A> Signal<A> for Branch<A> where
    A: 'static + Send + Clone,
{
    fn initial(&self) -> SignalType<A> {
        SignalType::Dynamic(self.initial.clone())
    }

    fn push_to(self: Box<Self>, target: Option<Box<Push<A>>>) {
        match (target, self.source_rx) {
            (Some(mut t), Some(rx)) => {
                debug!("Branch::push_to with target");

                loop {
                    match rx.recv() {
                        Ok(event) => t.push(event),
                        Err(_) => return,
                    }
                }
            },
            (None, Some(rx)) => {
                debug!("Branch::push_to with empty target");

                // Just ensuring the channel is drained so we don't get memory leaks
                loop {
                    match rx.recv() {
                        Err(_) => return,
                        _ => {},
                    }
                }
            },
            (Some(_), None) => {
                debug!("Branch::push_to with no source")
            },

            (None, None) => {
                debug!("Branch::push_to with neither target nor source")
            },

        }
    }

    fn init(&mut self) {
        let (tx, rx) = channel();
        self.fork_txs.lock().unwrap().push(tx);
        self.source_rx = Some(rx);
    }
}

impl<A> Clone for Branch<A> where
    A: 'static + Send + Clone,
{
    fn clone(&self) -> Branch<A> {
        Branch { fork_txs: self.fork_txs.clone(), source_rx: None, initial: self.initial.clone() }
    }
}
