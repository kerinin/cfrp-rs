mod channel;
mod fold;
mod fork;
mod input;
mod lift;
mod reactive;
mod topology;

use std::sync::*;
use std::sync::mpsc::*;

pub use reactive::Reactive;
pub use topology::{Topology, Builder};

#[derive(Clone)]
pub enum Event<A> {
    Changed(A),
    Unchanged,
    NoOp,
    Exit,
}

pub trait Signal<A>
{
    fn pull(&mut self) -> Event<A>;
}

trait Run: Send {
    fn run(mut self: Box<Self>);
}

pub struct Channel<A> where
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

pub struct Lift<F, A, B> where
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

pub struct Fold<F, A, B> where
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


pub struct Branch<A> where
    A: 'static + Send,
{
    // Arc<T> is send if T: Send + Sync (which mutex is, unconditionally)
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
            let fold = lift.foldp(out_tx, |tx, a| { tx.send(a); });
            t.add(Box::new(fold));

            // t.add(Box::new(
            //     plus_one.
            //         lift(|i: &usize| -> usize { i + 1 })
            // ));
        }).run();

        in_tx.send(0usize);

        println!("Received {}", out_rx.recv().unwrap());
    }
}
