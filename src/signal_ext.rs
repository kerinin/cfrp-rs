use super::{Signal};
use primitives::{LiftSignal, Lift2Signal, FoldSignal, Builder};

pub trait AsSignal<A> where
A: 'static + Send + Clone
{
    fn as_signal(self: Box<Self>) -> Box<Signal<A>>;
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
pub trait SignalExt<A> where
A: 'static + Send + Clone,
{
    fn lift<F, B>(mut self, f: F) -> Box<Signal<B>> where
    F: 'static + Send + Fn(A) -> B,
    B: 'static + Send + Clone;

    fn lift2<F, B, C>(mut self, mut right: Box<AsSignal<B>>, f: F) -> Box<Signal<C>> where
    F: 'static + Send + Fn(A, B) -> C,
    B: 'static + Send + Clone,
    C: 'static + Send + Clone;

    fn fold<F, B>(mut self, mut initial: B, mut f: F) -> Box<Signal<B>> where
    F: 'static + Send + FnMut(&mut B, A),
    B: 'static + Send + Clone;

    fn add_to(self, builder: &Builder) -> Box<Signal<A>>;
}

impl<A> SignalExt<A> for Box<Signal<A>> where
A: 'static + Send + Clone,
{
    fn lift<F, B>(mut self, f: F) -> Box<Signal<B>> where
    F: 'static + Send + Fn(A) -> B,
    B: 'static + Send + Clone,
    {
        self.init();

        Box::new(LiftSignal::new(self, f))
    }

    fn lift2<F, B, C>(mut self, raw_right: Box<AsSignal<B>>, f: F) -> Box<Signal<C>> where
    F: 'static + Send + Fn(A, B) -> C,
    B: 'static + Send + Clone,
    C: 'static + Send + Clone,
    {
        let mut right = raw_right.as_signal();
        self.init();
        right.init();

        Box::new(Lift2Signal::new(self, right, f))
    }

    fn fold<F, B>(mut self, initial: B, f: F) -> Box<Signal<B>> where
    F: 'static + Send + FnMut(&mut B, A),
    B: 'static + Send + Clone,
    {
        self.init();

        Box::new(FoldSignal::new(self, initial, f))
    }

    fn add_to(self, builder: &Builder) -> Box<Signal<A>> {
        builder.add(self)
    }
}
