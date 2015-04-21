use super::{Signal, Lift, Fold};

/// Methods for manipulating data in a topology
///
pub trait SignalExt<A> {
    fn lift<F, B>(self, f: F) -> Lift<F, A, B> where
        F: 'static + Send + Fn(A) -> B,
        A: 'static + Send,
        B: 'static + Send;

    fn foldp<F, B>(self, initial: B, f: F) -> Fold<F, A, B> where
        F: 'static + Send + FnMut(&mut B, A),
        A: 'static + Send,
        B: 'static + Send + Clone;
}

impl<A, T> SignalExt<A> for T where T: 'static + Signal<A> + Send
{
    fn lift<F, B>(self, f: F) -> Lift<F, A, B> where
        F: 'static + Send + Fn(A) -> B,
        A: 'static + Send,
        B: 'static + Send,
    {
        Lift::new(Box::new(self), f)
    }

    fn foldp<F, B>(self, initial: B, f: F) -> Fold<F, A, B> where
        F: 'static + Send + FnMut(&mut B, A),
        A: 'static + Send,
        B: 'static + Send + Clone,
    {
        Fold::new(Box::new(self), f, initial)
    }
}
