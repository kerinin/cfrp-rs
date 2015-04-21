use super::{Signal, Lift, Fold};

/// Methods for manipulating data in a topology
///
pub trait SignalExt<A> {
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
    fn lift<F, B>(self: Box<Self>, f: F) -> Box<Signal<B>> where
        F: 'static + Send + Fn(A) -> B,
        A: 'static + Send,
        B: 'static + Send;

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
    fn foldp<F, B>(self: Box<Self>, initial: B, f: F) -> Box<Signal<B>> where
        F: 'static + Send + FnMut(&mut B, A),
        A: 'static + Send,
        B: 'static + Send + Clone;
}

impl<A, T> SignalExt<A> for T where T: 'static + Signal<A> + Send
{
    fn lift<F, B>(self: Box<Self>, f: F) -> Box<Signal<B>> where
        F: 'static + Send + Fn(A) -> B,
        A: 'static + Send,
        B: 'static + Send,
    {
        Box::new(Lift::new(self, f))
    }

    fn foldp<F, B>(self: Box<Self>, initial: B, f: F) -> Box<Signal<B>> where
        F: 'static + Send + FnMut(&mut B, A),
        A: 'static + Send,
        B: 'static + Send + Clone,
    {
        Box::new(Fold::new(self, f, initial))
    }
}
