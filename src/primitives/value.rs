use super::super::{Signal, SignalExt, SignalType, Push, Config};

#[derive(Clone)]
pub struct Value<A> where
    A: Send + Clone,
{
    config: Config,
    initial: A,
}

impl<A> Value<A> where 
    A: Send + Clone
{
    pub fn new(config: Config, v: A) -> Self {
        Value { config: config, initial: v }
    }
}

impl<A> Signal<A> for Value<A> where
    A: 'static + Send + Clone,
{
    fn config(&self) -> Config {
        self.config.clone()
    }

    fn initial(&self) -> SignalType<A> {
        SignalType::Constant(self.initial.clone())
    }

    fn push_to(self: Box<Self>, _: Option<Box<Push<A>>>) {
        panic!("Constant-typed signal asked to push - stack overflows ahoy!")
    }
}
impl<A> SignalExt<A> for Value<A> where
    A: 'static + Send + Clone,
{}
