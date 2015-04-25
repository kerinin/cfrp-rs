use std::sync::*;
use std::sync::mpsc::*;

use super::super::{Event, Signal, Push, Lift, Lift2, Fold};
use super::lift::*;
use super::lift2::*;
use super::fold::*;

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
impl<A> Signal<A> for Branch<A> where
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

impl<A> Lift<A> for Branch<A> where
    A: 'static + Send,
{
    fn lift<F, B>(mut self, f: F) -> LiftSignal<F, A, B> where
        F: 'static + Send + Fn(A) -> B,
        A: 'static + Send,
        B: 'static + Send,
    {
        let (tx, rx) = channel();
        self.fork_txs.lock().unwrap().push(tx);
        self.source_rx = Some(rx);

        LiftSignal {
            parent: Box::new(self),
            f: f,
        }
    }
}

impl<A, B, SB> Lift2<A, B, SB> for Branch<A> where
    A: 'static + Send,
{
    fn lift2<F, C>(mut self, right: SB, f: F) -> Lift2Signal<F, A, B, C> where
        Self: 'static,
        SB: 'static + Signal<B>,
        F: 'static + Send + Fn(Option<A>, Option<B>) -> C,
        A: 'static + Send + Clone,
        B: 'static + Send + Clone,
        C: 'static + Send + Clone,
    {
        let (tx, rx) = channel();
        self.fork_txs.lock().unwrap().push(tx);
        self.source_rx = Some(rx);

        Lift2Signal {
            left: Box::new(self),
            right: Box::new(right),
            f: f,
        }
    }
}

impl<A> Fold<A> for Branch<A> where
    A: 'static + Send,
{
    fn fold<F, B>(mut self, initial: B, f: F) -> FoldSignal<F, A, B> where
        F: 'static + Send + FnMut(&mut B, A),
        A: 'static + Send + Clone,
        B: 'static + Send + Clone,
    {
        let (tx, rx) = channel();
        self.fork_txs.lock().unwrap().push(tx);
        self.source_rx = Some(rx);

        FoldSignal {
            parent: Box::new(self),
            f: f,
            state: initial,
        }
    }
}
