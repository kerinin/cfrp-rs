use super::{Signal, Run};

pub struct Lift<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{
    parent: Box<Signal<A> + Send>,
    f: F,
}

impl<F, A, B> Lift<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{
    pub fn new(parent: Box<Signal<A> + Send>, f: F) -> Lift<F, A, B> {
        Lift {
            parent: parent,
            f: f,
        }
    }
}

impl<F, A, B> Signal<B> for Lift<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{
    fn recv(&self) -> Option<B> {
       match self.parent.recv() {
           Some(a) => Some((self.f)(a)),
           None => None,
       }
    }
}

impl<F, A, B> Run for Lift<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{
    fn run(self: Box<Self>) {
        loop {
            self.recv();
        }
    }
}
