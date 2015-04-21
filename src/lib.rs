mod channel;
mod fold;
mod fork;
mod input;
mod lift;
mod signal;
mod topology;

use std::sync::*;
use std::sync::mpsc::*;

pub use signal::SignalExt;
pub use topology::{Topology, Builder};

#[derive(Clone)]
pub enum Event<A> {
    Changed(A),
    Unchanged,
    NoOp,
    Exit,
}

/// Types which can be used as nodes in a topology
pub trait Signal<A>
{
    fn push_to(self: Box<Self>, Box<Push<A>>);
}

pub trait Push<A> {
    fn push(&mut self, Event<A>);
}

trait Run: Send {
    fn run(mut self: Box<Self>);
}

struct Channel<A> where
    A: 'static + Send,
{
    source_rx: Receiver<Event<A>>,
}

impl<A> Channel<A> where
    A: 'static + Send,
{
    fn new(source_rx: Receiver<Event<A>>) -> Channel<A> {
        Channel {
            source_rx: source_rx,
        }
    }
}

struct Lift<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{
    parent: Box<Signal<A> + Send>,
    f: F,
}

impl<F, A, B> Lift<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{
    fn new(parent: Box<Signal<A> + Send>, f: F) -> Lift<F, A, B> {
        Lift {
            parent: parent,
            f: f,
        }
    }
}

struct Fold<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    parent: Box<Signal<A> + Send>,
    f: F,
    state: B,
}

impl<F, A, B> Fold<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    fn new(parent: Box<Signal<A> + Send>, f: F, initial: B) -> Fold<F, A, B> {
        Fold {
            parent: parent,
            f: f,
            state: initial,
        }
    }
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
struct Fork<A> where
    A: 'static + Send,
{
    parent: Box<Signal<A> + Send>,
    sink_txs: Arc<Mutex<Vec<Sender<Event<A>>>>>,
}

impl<A> Fork<A> where
    A: 'static + Clone + Send,
{
    fn new(parent: Box<Signal<A> + Send>, sink_txs: Arc<Mutex<Vec<Sender<Event<A>>>>>) -> Fork<A> {
        Fork {
            parent: parent,
            sink_txs: sink_txs,
        }
    }
}


/// `Branch<A>` allows a data source `A` to be used as input more than once
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
    source_rx: Receiver<Event<A>>,
}

impl<A> Branch<A> where
    A: 'static + Send,
{
    fn new(fork_txs: Arc<Mutex<Vec<Sender<Event<A>>>>>, source_rx: Receiver<Event<A>>) -> Branch<A> {
        Branch {
            fork_txs: fork_txs,
            source_rx: source_rx,
        }
    }
}



#[cfg(test)] 
mod test {
    // extern crate quickcheck;
    use std::sync::mpsc::*;
    
    use super::*;

    #[test]
    fn integration() {
        let (in_tx, in_rx) = channel();
        let (out_tx, out_rx) = channel();

        Topology::build( (in_rx, out_tx), |t, (in_rx, out_tx)| {

            let channel = t.channel(in_rx);
            let lift = channel.lift(|i| -> usize { i + 1 });

            let plus_one = t.add(t.channel(in_rx)
                .lift(|i| -> usize { i + 1 })
            );

            let plus_two = t.add(plus_one
                .lift(|i| -> usize { i + 1 })
            );

            t.add(plus_two
                .foldp(out_tx, |tx, a| { tx.send(a); })
            );

        }).run();

        in_tx.send(0usize);

        let out = out_rx.recv().unwrap();
        assert_eq!(out, 2);
        println!("Received {}", out);
    }
}
