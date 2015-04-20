use super::{Lift, Signal, Run};

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
