use super::{Signal, Builder};
use primitives::lift::LiftSignal;
use primitives::lift2::Lift2Signal;
use primitives::fold::FoldSignal;
use primitives::fork::Branch;

pub trait AsSignal<A> where
A: 'static + Send + Clone
{
    fn as_signal(self) -> Box<Signal<A>>;
}

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
