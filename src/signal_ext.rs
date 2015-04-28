use super::{Signal};
use primitives::{LiftSignal, Lift2Signal, FoldSignal, Builder, Branch};

pub trait AsSignal<A> where
A: 'static + Send + Clone
{
    fn as_signal(self) -> Box<Signal<A>>;
}

//
// * We don't know the return value of some functions so we must return a trait object
// * Some functions should accept both a "signal" and a "signals"
//
// # Type Object approach
//
// * Return Signal/Signals (so we can return multiple types)
// * Implement AsSignal for Signal/Signals (so we can accept both Signal and Signals)
// * Implement SignalExt for Signal/Signals (so we can call both Signal and Signals)
// * Use trait objects as method args
//
//
// # Concrete type approach
//
// * Return enumerations (so we can return multiple types)
// * Implement Signal for return types
// * Implement SignalExt for Signal
// * Use typed objects as method args
//
// Upside is this is less complex conceptually.  Donwnside is that it makes Liftn
// harder (enumeration variants increases exponenially with N).  Alternately, 
// we can push the constant-value optimization into LiftN itself, rather than
// changing the return type.
//
pub trait SignalExt<A>: Signal<A> + Sized where
Self: 'static,
A: 'static + Send + Clone,
{
    fn lift<F, B>(mut self, f: F) -> LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    B: 'static + Send + Clone,
    {
        self.init();

        LiftSignal::new(Box::new(self), f)
    }

    fn lift2<F, SB, B, C>(mut self, mut right: SB, f: F) -> Lift2Signal<F, A, B, C> where
    SB: 'static + Signal<B>,
    F: 'static + Send + Fn(A, B) -> C,
    B: 'static + Send + Clone,
    C: 'static + Send + Clone,
    {
        self.init();
        right.init();

        Lift2Signal::new(Box::new(self), Box::new(right), f)
    }

    fn fold<F, B>(mut self, initial: B, f: F) -> FoldSignal<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    B: 'static + Send + Clone,
    {
        self.init();

        FoldSignal::new(Box::new(self), initial, f)
    }

    fn add_to(self, builder: &Builder) -> Branch<A> {
        builder.add(self)
    }
}
