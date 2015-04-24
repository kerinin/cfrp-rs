pub mod channel;
pub mod fold;
pub mod fork;
pub mod input;
pub mod lift;
// pub mod liftn;
pub mod lift2;

#[derive(Clone)]
pub enum Event<A> {
    Changed(A),
    Unchanged,
    NoOp,
    Exit,
}

pub trait InternalSignal<A>: Send
{
    fn push_to(self: Box<Self>, Option<Box<Push<A>>>);
}

pub trait Push<A> {
    fn push(&mut self, Event<A>);
}

pub trait Run: Send {
    fn run(mut self: Box<Self>);
}
