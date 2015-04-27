use super::{Signal, SignalType};
use primitives::{Value, LiftSignal, Lift2Signal, FoldSignal, Builder, Branch};

pub trait SignalExt<A> where
A: 'static + Send + Clone,
{
    fn lift<F, B>(mut self, f: F) -> Box<Signal<B>> where
    F: 'static + Send + Fn(A) -> B,
    B: 'static + Send + Clone;

    fn lift2<F, B, C>(mut self, mut right: Box<Signal<B>>, f: F) -> Box<Signal<C>> where
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

        match self.initial() {
            SignalType::Dynamic(v) => {
                let initial = f(v);
                Box::new(LiftSignal::new(self, f, initial))
            }
            SignalType::Constant(v) => {
                let initial = f(v);
                Box::new(Value::new(initial))
            }
        }
    }

    fn lift2<F, B, C>(mut self, mut right: Box<Signal<B>>, f: F) -> Box<Signal<C>> where
    F: 'static + Send + Fn(A, B) -> C,
    B: 'static + Send + Clone,
    C: 'static + Send + Clone,
    {
        self.init();
        right.init();

        match (self.initial(), right.initial()) {
            (SignalType::Dynamic(l), SignalType::Dynamic(r)) => {
                let initial = f(l,r);
                Box::new(Lift2Signal::new(self, right, f, initial))
            }

            (SignalType::Dynamic(l), SignalType::Constant(r)) => {
                let initial = f(l, r.clone());
                let signal = LiftSignal::new(
                    self,
                    move |dynamic_l| -> C { f(dynamic_l, r.clone()) },
                    initial,
                    );

                Box::new(signal)
            }

            (SignalType::Constant(l), SignalType::Dynamic(r)) => {
                let initial = f(l.clone(), r);
                let signal = LiftSignal::new(
                    right,
                    move |dynamic_r| -> C { f(l.clone(), dynamic_r) },
                    initial,
                    );

                Box::new(signal)
            }

            (SignalType::Constant(l), SignalType::Constant(r)) => {
                let initial = f(l, r);
                Box::new(Value::new(initial))
            }
        }
    }

    fn fold<F, B>(mut self, mut initial: B, mut f: F) -> Box<Signal<B>> where
    F: 'static + Send + FnMut(&mut B, A),
    B: 'static + Send + Clone,
    {
        self.init();

        match self.initial() {
            SignalType::Dynamic(v) => {
                f(&mut initial, v);
                Box::new(FoldSignal::new(self, initial, f))
            }

            SignalType::Constant(v) => {
                f(&mut initial, v);
                Box::new(Value::new(initial))
            }
        }
    }

    fn add_to(self, builder: &Builder) -> Box<Signal<A>> {
        builder.add(self)
    }
}

impl<A> SignalExt<A> for Box<Branch<A>> where
A: 'static + Send + Clone,
{
    fn lift<F, B>(mut self, f: F) -> Box<Signal<B>> where
    F: 'static + Send + Fn(A) -> B,
    B: 'static + Send + Clone,
    {
        self.init();

        match self.initial() {
            SignalType::Dynamic(v) => {
                let initial = f(v);
                Box::new(LiftSignal::new(self, f, initial))
            }
            SignalType::Constant(v) => {
                let initial = f(v);
                Box::new(Value::new(initial))
            }
        }
    }

    fn lift2<F, B, C>(mut self, mut right: Box<Signal<B>>, f: F) -> Box<Signal<C>> where
    F: 'static + Send + Fn(A, B) -> C,
    B: 'static + Send + Clone,
    C: 'static + Send + Clone,
    {
        self.init();
        right.init();

        match (self.initial(), right.initial()) {
            (SignalType::Dynamic(l), SignalType::Dynamic(r)) => {
                let initial = f(l,r);
                Box::new(Lift2Signal::new(self, right, f, initial))
            }

            (SignalType::Dynamic(l), SignalType::Constant(r)) => {
                let initial = f(l, r.clone());
                let signal = LiftSignal::new(
                    self,
                    move |dynamic_l| -> C { f(dynamic_l, r.clone()) },
                    initial,
                    );

                Box::new(signal)
            }

            (SignalType::Constant(l), SignalType::Dynamic(r)) => {
                let initial = f(l.clone(), r);
                let signal = LiftSignal::new(
                    right,
                    move |dynamic_r| -> C { f(l.clone(), dynamic_r) },
                    initial,
                    );

                Box::new(signal)
            }

            (SignalType::Constant(l), SignalType::Constant(r)) => {
                let initial = f(l, r);
                Box::new(Value::new(initial))
            }
        }
    }

    fn fold<F, B>(mut self, mut initial: B, mut f: F) -> Box<Signal<B>> where
    F: 'static + Send + FnMut(&mut B, A),
    B: 'static + Send + Clone,
    {
        self.init();

        match self.initial() {
            SignalType::Dynamic(v) => {
                f(&mut initial, v);
                Box::new(FoldSignal::new(self, initial, f))
            }

            SignalType::Constant(v) => {
                f(&mut initial, v);
                Box::new(Value::new(initial))
            }
        }
    }

    fn add_to(self, builder: &Builder) -> Box<Signal<A>> {
        builder.add(self)
    }
}
