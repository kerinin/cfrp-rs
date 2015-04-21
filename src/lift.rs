use super::{Lift, Signal, Run};

impl<F, A, B> Signal<B> for Lift<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{
    fn recv(&mut self) -> Option<B> {
        let received = self.parent.recv();
        match received {
            Some(a) => {
                let b = (self.f)(a);
                Some(b)
            },
            None => None,
        }
    }
}

impl<F, A, B> Run for Lift<F, A, B> where
F: 'static + Send + Fn(A) -> B,
A: 'static + Send,
B: 'static + Send,
{
    fn run(mut self: Box<Self>) {
        loop {
            self.recv();
        }
    }
}
