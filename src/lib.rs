mod channel;
mod fold;
mod fork;
mod input;
mod lift;
mod signal;
mod topology;
mod liftn;
mod lift2;

use std::sync::*;
use std::sync::mpsc::*;

pub use topology::{Topology, Builder};

#[derive(Clone)]
pub enum Event<A> {
    Changed(A),
    Unchanged,
    NoOp,
    Exit,
}

/// A data source of type `A`
///
pub struct Signal<A> {
    internal_signal: Box<InternalSignal<A>>,
}

pub trait InternalSignal<A>: Send
{
    fn push_to(self: Box<Self>, Option<Box<Push<A>>>);
}

pub trait Push<A> {
    fn push(&mut self, Event<A>);
}

trait Run: Send {
    fn run(mut self: Box<Self>);
}

trait RunInput: Send {
    fn run(mut self: Box<Self>, usize, Arc<Mutex<Vec<Box<NoOp>>>>);
    fn boxed_no_op(&self) -> Box<NoOp>;
}

pub trait Input<A> {
    fn pull(&mut self) -> Option<A>;
}

trait NoOp: Send {
    fn send_no_change(&self);
    fn send_exit(&self);
}

struct InternalInput<A> where
    A: 'static + Send + Clone
{
    input: Box<Input<A> + Send>,
    sink_tx: Sender<Event<A>>,
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
    parent: Box<InternalSignal<A>>,
    f: F,
}

pub trait InputList<Head> {
    type InputPullers: 'static + PullInputs;

    fn run(Head, Self) -> Self::InputPullers;
}

trait PullInputs {
    type Values;

    fn pull(&mut self, any_changed: &mut bool, any_exit: &mut bool) -> Self::Values;
}

struct Lift2<F, A, B, C> where
    F: 'static + Send + Fn(Option<A>, Option<B>) -> C,
    A: 'static + Send,
    B: 'static + Send,
    C: 'static + Send + Clone,
{
    left: Box<InternalSignal<A>>,
    right: Box<InternalSignal<B>>,
    f: F,
}

struct LiftN<F, A, R, B> where
    F: Fn(<<R as InputList<A>>::InputPullers as PullInputs>::Values) -> B,
    R: InputList<A>,
{
    head: A,
    rest: R,
    f: F,
}

struct Fold<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    A: 'static + Send,
    B: 'static + Send + Clone,
{
    parent: Box<InternalSignal<A>>,
    f: F,
    state: B,
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
    parent: Box<InternalSignal<A>>,
    sink_txs: Arc<Mutex<Vec<Sender<Event<A>>>>>,
}

impl<A> Fork<A> where
    A: 'static + Clone + Send,
{
    fn new(parent: Box<InternalSignal<A>>, sink_txs: Arc<Mutex<Vec<Sender<Event<A>>>>>) -> Fork<A> {
        Fork {
            parent: parent,
            sink_txs: sink_txs,
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
    fn new(fork_txs: Arc<Mutex<Vec<Sender<Event<A>>>>>, source_rx: Option<Receiver<Event<A>>>) -> Branch<A> {
        Branch {
            fork_txs: fork_txs,
            source_rx: source_rx,
        }
    }
}



#[cfg(test)] 
mod test {
    extern crate log;

    // extern crate quickcheck;
    use std::thread;
    use std::sync::mpsc::*;

    use super::*;

    #[test]
    fn integration() {
        let (in_tx, in_rx) = channel();
        let (out_tx, out_rx) = channel();

        Topology::build( (in_rx, out_tx), |t, (in_rx, out_tx)| {

            let input = t.add(t.listen(in_rx));

            // t.add(input.clone()
            //       .liftn((input,), |(i, j)| -> usize { println!("lifting"); 0 })
            //       .foldp(out_tx.clone(), |tx, a| { tx.send(a); })
            //      );
            t.add(input.clone()
                  .lift(|i| -> usize { i })
                  .lift2(input.lift(|i| -> usize { i }), |i, j| -> usize {
                      println!("lifting");
                      match (i, j) {
                          (Some(a), Some(b)) => a + b,
                          _ => 0,
                      } 
                  }).foldp(out_tx.clone(), |tx, a| { tx.send(a); })
                 );





            /*
            let plus_one = t.add(t.listen(in_rx)
                .lift(|i| -> usize { println!("lifting to plus_one"); i + 1 })
            );

            let plus_two = t.add(plus_one.clone()
                .lift(|i| -> usize { println!("lifting to plus_two"); i + 1 })
            );

            let plus_three = t.add(plus_one.clone()
                .lift(|i| -> usize { println!("lifting to plus_three"); i + 2 })
            );

            t.add(plus_two
                .liftn((plus_three,), |(i, j)| -> usize { 
                    println!("liftn-ing to lifted");

                    match (i, j) {
                        (Some(a), Some(b)) => { a + b },
                        _ => 0,
                    }
                }).foldp(out_tx, |tx, a| { tx.send(a); })
            );

            t.add(plus_two
                .foldp(out_tx.clone(), |tx, a| { tx.send(a); })
            );
            t.add(plus_three
                .foldp(out_tx.clone(), |tx, a| { tx.send(a); })
            );
            */

        }).run();

        thread::sleep_ms(1000);

        in_tx.send(1usize);

        let out = out_rx.recv().unwrap();
        assert_eq!(out, 2);
        // println!("Received {}", out);
        /*

        let out = out_rx.recv().unwrap();
        assert_eq!(out, 3);
        println!("Received {}", out);
        */
    }
}
