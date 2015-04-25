pub mod primitives;
mod topology;

pub use topology::{Topology, Builder};
use primitives::lift::LiftSignal;
use primitives::lift2::Lift2Signal;
use primitives::fold::FoldSignal;

#[derive(Clone)]
pub enum Event<A> {
    Changed(A),
    Unchanged,
    NoOp,
    Exit,
}

pub trait Signal<A>: Send {
    // Called at build time when a downstream process is created for the signal
    fn init(&mut self) {}

    // Called at compile time when a donstream process is run
    fn push_to(self: Box<Self>, Option<Box<Push<A>>>);
}

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
pub trait Lift<A>: Signal<A> + Sized {
    fn lift<F, B>(mut self, f: F) -> LiftSignal<F, A, B> where
        Self: 'static,
        F: 'static + Send + Fn(A) -> B,
        A: 'static + Send,
        B: 'static + Send,
    {
        self.init();

        LiftSignal {
            parent: Box::new(self),
            f: f,
        }
    }
}

pub trait Lift2<A, B, SB>: Signal<A> + Sized {
    fn lift2<F, C>(mut self, mut right: SB, f: F) -> Lift2Signal<F, A, B, C> where
        Self: 'static,
        SB: 'static + Signal<B>,
        F: 'static + Send + Fn(Option<A>, Option<B>) -> C,
        A: 'static + Send + Clone,
        B: 'static + Send + Clone,
        C: 'static + Send + Clone,
    {
        self.init();
        right.init();

        Lift2Signal {
            left: Box::new(self),
            right: Box::new(right),
            f: f,
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
pub trait Fold<A>: Signal<A> + Sized {
    fn fold<F, B>(mut self, initial: B, f: F) -> FoldSignal<F, A, B> where
        Self: 'static,
        F: 'static + Send + FnMut(&mut B, A),
        A: 'static + Send + Clone,
        B: 'static + Send + Clone,
    {
        self.init();

        FoldSignal {
            parent: Box::new(self),
            f: f,
            state: initial,
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
            //       .fold(out_tx.clone(), |tx, a| { tx.send(a); })
            //      );
            t.add(input.clone()
                  .lift(|i| -> usize { i })
                  .lift2(input, |i, j| -> usize {
                      println!("lifting");
                      match (i, j) {
                          (Some(a), Some(b)) => a + b,
                          _ => 0,
                      } 
                  })
                  .fold(out_tx.clone(), |tx, a| { tx.send(a); })
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
                }).fold(out_tx, |tx, a| { tx.send(a); })
            );

            t.add(plus_two
                .fold(out_tx.clone(), |tx, a| { tx.send(a); })
            );
            t.add(plus_three
                .fold(out_tx.clone(), |tx, a| { tx.send(a); })
            );
            */

        }).run();

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
