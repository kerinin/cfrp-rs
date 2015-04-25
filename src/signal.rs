use super::{Signal, Lift, Fold};
use super::primitives::lift::*;
use super::primitives::lift2::*;
use super::primitives::fold::*;

impl<A> Lift<A> for Signal<A>
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
    fn lift<F, B>(self, f: F) -> Signal<B> where
        F: 'static + Send + Fn(A) -> B,
        A: 'static + Send,
        B: 'static + Send,
    {
        Signal {
            internal_signal: Box::new(
                LiftSignal {
                    parent: self.internal_signal,
                    f: f,
                }
            ),
        }
    }
}

/*
impl<A, SB, B> Lift2<SB> for Signal<A> where SB: Signal<B>
{
    pub fn lift2<F, C>(self, right: SB, f: F) -> Signal<C> where
        F: 'static + Send + Fn(Option<A>, Option<B>) -> C,
        A: 'static + Send + Clone,
        B: 'static + Send + Clone,
        C: 'static + Send + Clone,
    {
        Signal {
            internal_signal: Box::new(
                Lift2Signal {
                    left: self.internal_signal,
                    right: right.internal_signal,
                    f: f,
                }
            )
        }
    }
}
*/

impl<A> Fold<A> for Signal<A>
{
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
    fn foldp<F, B>(self, initial: B, f: F) -> Signal<B> where
        F: 'static + Send + FnMut(&mut B, A),
        A: 'static + Send + Clone,
        B: 'static + Send + Clone,
    {
        Signal {
            internal_signal: Box::new(
                FoldSignal {
                    parent: self.internal_signal,
                    f: f,
                    state: initial,
                }
            )
        }
    }
}
