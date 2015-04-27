use super::super::{Signal, SignalType, Push};

#[derive(Clone)]
pub struct Value<A> where
    A: Send + Clone,
{
    initial: A,
}

impl<A> Value<A> where 
    A: Send + Clone
{
    pub fn new(v: A) -> Self {
        Value { initial: v }
    }
}

impl<A> Signal<A> for Value<A> where
    A: 'static + Send + Clone,
{
    fn initial(&self) -> SignalType<A> {
        SignalType::Constant(self.initial.clone())
    }

    fn push_to(self: Box<Self>, _: Option<Box<Push<A>>>) {
        panic!("Constant-typed signal asked to push - stack overflows ahoy!")
    }
}
