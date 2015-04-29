use super::{Signal, Builder};
use primitives::lift::LiftSignal;
use primitives::lift2::Lift2Signal;
use primitives::fold::FoldSignal;
use primitives::fork::Branch;

pub trait SignalExt<A>: Signal<A> + Sized where
Self: 'static,
A: 'static + Send + Clone,
{
    fn lift<F, B>(mut self, f: F) -> LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    B: 'static + Send + Clone,
    {
        self.init();

        LiftSignal::new(self.config(), Box::new(self), f)
    }

    fn lift2<F, SB, B, C>(mut self, mut right: SB, f: F) -> Lift2Signal<F, A, B, C> where
    SB: 'static + Signal<B>,
    F: 'static + Send + Fn(A, B) -> C,
    B: 'static + Send + Clone,
    C: 'static + Send + Clone,
    {
        self.init();
        right.init();

        Lift2Signal::new(self.config(), Box::new(self), Box::new(right), f)
    }

    fn fold<F, B>(mut self, initial: B, f: F) -> FoldSignal<F, A, B> where
    F: 'static + Send + FnMut(&mut B, A),
    B: 'static + Send + Clone,
    {
        self.init();

        FoldSignal::new(self.config(), Box::new(self), initial, f)
    }

    fn add_to(self, builder: &Builder) -> Branch<A> {
        builder.add(self)
    }

    fn async(self, builder: &Builder) -> Branch<A> {
        builder.async(self)
    }

    fn map<F, B>(self, f: F) -> LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    B: 'static + Send + Clone,
    {
        self.lift(f)
    }

    fn zip<SB, B>(self, right: SB) -> Box<Signal<(A, B)>> where
    SB: 'static + Signal<B>,
    B: 'static + Send + Clone,
    {
        Box::new(
            self.lift2(
                right,
                |l: A, r: B| -> (A, B) { (l,r) }
            )
        )
    }

    fn enumerate(self) -> Box<Signal<(usize, A)>>
    {
        let initial = self.initial().unwrap();
        Box::new(
            self.fold(
                (0, initial),
                |state: &mut (usize, A), i: A| { *state = (state.0 + 1, i) },
            )
        )
    }

    fn filter<F>(self, f: F) -> Box<Signal<Option<A>>> where
    F: 'static + Send + Fn(&A) -> bool,
    {
        Box::new(
            self.lift(move |i| {
                if f(&i) {
                    Some(i)
                } else {
                    None
                }
            })
        )
    }

    fn inspect(self, f: F)
    F: 'static + Send + Fn(&A) -> bool,
    {
        Box::new(
            self.lift(move |i| {
                f(i);
                i
            })
        }
    }
}
