use super::{Signal, Builder};
use primitives::lift::LiftSignal;
use primitives::lift2::Lift2Signal;
use primitives::fold::FoldSignal;
use primitives::fork::Branch;

pub trait SignalExt<A>: Signal<A> + Sized where
    Self: 'static,
    A: 'static + Send + Clone,
{
    /// Transform in input signal into an output signal
    ///
    /// # Example
    ///
    /// ```
    /// use std::default::Default;
    /// use std::sync::mpsc::*;
    /// use cfrp::*;
    ///
    /// let (in_tx, in_rx) = sync_channel(0);
    /// let (out_tx, out_rx) = channel();
    ///
    /// spawn_topology(Default::default(), move |t| {
    ///     t.listen(0, in_rx)
    ///         .lift(move |i| { out_tx.send(i | (1 << 1)).unwrap(); })
    ///         .add_to(t);
    /// });
    ///
    /// // Initial value
    /// assert_eq!(out_rx.recv().unwrap(), 0b00000010);
    ///
    /// // Lifted value
    /// in_tx.send(1).unwrap();
    /// assert_eq!(out_rx.recv().unwrap(), 0b00000011);
    /// ```
    ///
    fn lift<F, B>(mut self, f: F) -> LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    B: 'static + Send + Clone,
    {
        self.init();

        LiftSignal::new(self.config(), Box::new(self), f)
    }

    /// Combine two signals into an output signal
    ///
    /// # Example
    ///
    /// ```
    /// use std::default::Default;
    /// use std::sync::mpsc::*;
    /// use cfrp::*;
    ///
    /// let (l_tx, l_rx) = sync_channel(0);
    /// let (r_tx, r_rx) = sync_channel(0);
    /// let (out_tx, out_rx) = channel();
    ///
    /// spawn_topology(Default::default(), move |t| {
    ///     t.listen(1 << 0, l_rx)
    ///         .lift2(t.listen(1 << 1, r_rx), move |i,j| { out_tx.send(i | j).unwrap() })
    ///         .add_to(t);
    /// });
    ///
    /// // Initial value
    /// assert_eq!(out_rx.recv().unwrap(), (1 << 0) | (1 << 1));
    ///
    /// l_tx.send(1 << 2).unwrap();
    /// assert_eq!(out_rx.recv().unwrap(), (1 << 2) | (1 << 1));
    ///
    /// r_tx.send(1 << 3).unwrap();
    /// assert_eq!(out_rx.recv().unwrap(), (1 << 2) | (1 << 3));
    /// ```
    ///
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

    /// Merge data from a signal into an accumulator and return a signal with
    /// the accumulator's value
    ///
    /// # Example
    ///
    /// ```
    /// use std::default::Default;
    /// use std::sync::mpsc::*;
    /// use cfrp::*;
    ///
    /// let (in_tx, in_rx) = sync_channel(0);
    /// let (out_tx, out_rx) = channel();
    ///
    /// spawn_topology(Default::default(), move |t| {
    ///     t.listen(0, in_rx)
    ///         .fold(out_tx, |tx, i| { tx.send(i | (1 << 1)).unwrap(); tx })
    ///         .add_to(t);
    /// });
    ///
    /// // Initial value
    /// assert_eq!(out_rx.recv().unwrap(), 0b00000010);
    ///
    /// // Lifted value
    /// in_tx.send(1).unwrap();
    /// assert_eq!(out_rx.recv().unwrap(), 0b00000011);
    /// let (in_tx, in_rx) = sync_channel(0);
    /// let (out_tx, out_rx) = channel();
    ///
    /// spawn_topology(Default::default(), move |t| {
    ///     t.listen(0, in_rx)
    ///         .fold(out_tx, |tx, i| { tx.send(i | (1 << 1)).unwrap(); tx })
    ///         .add_to(t);
    /// });
    ///
    /// // Initial value
    /// assert_eq!(out_rx.recv().unwrap(), 0b00000010);
    ///
    /// // Lifted value
    /// in_tx.send(1).unwrap();
    /// assert_eq!(out_rx.recv().unwrap(), 0b00000011);
    ///
    fn fold<F, B>(mut self, initial: B, f: F) -> FoldSignal<F, A, B> where
    F: 'static + Send + Fn(B, A) -> B,
    B: 'static + Send + Clone,
    {
        self.init();

        FoldSignal::new(self.config(), Box::new(self), initial, f)
    }

    /// Sugar for `Builder::add`
    ///
    fn add_to(self, builder: &Builder) -> Branch<A> {
        builder.add(self)
    }

    /// Sugar for `Builder::async`
    ///
    fn async(self, builder: &Builder) -> Branch<A> {
        builder.async(self)
    }

    /// Alias of `lift`
    fn map<F, B>(self, f: F) -> LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    B: 'static + Send + Clone,
    {
        self.lift(f)
    }

    /// Takes two input signals and returns a signal containing 2-tuples
    /// of elements from the input signals.
    /// 
    /// Roughly equivalent to `Iterator::zip`, however if one signal
    /// changes but the other doesn't, the most-recent value of the unchanged
    /// signal will be output.  In other words, this operation doesn't block
    /// on receiving changes from both inputs.
    ///
    /// # Example
    ///
    /// ```
    /// use std::default::Default;
    /// use std::sync::mpsc::*;
    /// use cfrp::*;
    ///
    /// let (l_tx, l_rx): (SyncSender<usize>, Receiver<usize>) = sync_channel(0);
    /// let (r_tx, r_rx): (SyncSender<usize>, Receiver<usize>) = sync_channel(0);
    /// let (out_tx, out_rx) = channel();
    ///
    /// spawn_topology(Default::default(), move |t| {
    ///     t.listen(0, l_rx)
    ///         .zip(t.listen(0, r_rx))
    ///         .lift(move |i| { out_tx.send(i).unwrap(); })
    ///         .add_to(t);
    /// });
    ///
    /// // Initial value
    /// assert_eq!(out_rx.recv().unwrap(), (0, 0));
    ///
    /// l_tx.send(1).unwrap();
    /// assert_eq!(out_rx.recv().unwrap(), (1, 0));
    ///
    /// r_tx.send(1).unwrap();
    /// assert_eq!(out_rx.recv().unwrap(), (1, 1));
    /// ```
    ///
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
                |state: (usize, A), i: A| { (state.0 + 1, i) },
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

    fn inspect<F>(self, f: F) -> Box<Signal<A>> where
    F: 'static + Send + Fn(&A) -> bool,
    {
        Box::new(
            self.lift(move |i| {
                f(&i);
                i
            })
        )
    }
}
