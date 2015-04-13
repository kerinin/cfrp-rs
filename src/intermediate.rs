use std::thread;
use std::marker::*;
use std::sync::mpsc::*;

use super::runner::{LiftRunner, Lift2Runner, FoldRunner};

pub enum Event<T> {
    Changed(T),
    Same,
}



pub trait Signal<'a, A>: Parent<'a, A> {
    fn lift<F, B>(&mut self, f: F) -> LiftSignal<'a, F, A, B> 
    where F: 'static + Fn(&A) -> B + Clone + Send,
        A: 'static + Send,
        B: 'static + Send + Clone + Eq,
    {
        let (tx, rx) = channel();
        let signal = LiftSignal {f: f, rx: rx, outputs: Vec::new(), marker: PhantomData};
        // self.add_output(tx, &signal);
        signal
    }

    fn lift2<F, SB, B, C>(&mut self, b: &mut SB, f: F) -> Lift2Signal<'a, F, A, B, C>
    where F: 'static + Fn(&A, &B) -> C + Clone + Send,
        A: 'static + Send,
        B: 'static + Send,
        SB: Parent<'a, B>,
        C: 'static + Send + Clone + Eq,
    {
        let (tx_l, rx_l) = channel();
        let (tx_r, rx_r) = channel();
        let signal = Lift2Signal {f: f, rx_r: rx_r, rx_l: rx_l, outputs: Vec::new(), marker_a: PhantomData, marker_b: PhantomData, marker_c: PhantomData};
        // self.add_output(tx_l, &signal);
        // b.add_output(tx_r, &signal);
        signal
    }

    fn foldp<F, B>(&mut self, f: F, initial: B) -> FoldSignal<'a, F, A, B>
    where F: 'static + Fn(&B, &A) -> B + Clone + Send,
        A: 'static + Send,
        B: 'static + Send + Clone + Eq,
    {
        let (tx, rx) = channel();
        let signal = FoldSignal {f: f, rx: rx, state: initial, outputs: Vec::new(), marker_a: PhantomData, marker_b: PhantomData};
        // self.add_output(tx, &signal);
        signal
    }
}

trait Parent<'a, A> {
    fn add_output(&mut self, Sender<Event<A>>, &'a Child);
}

trait Child {
    fn start(&self);
}



pub struct Reactor;

impl Reactor {
    fn channel<'a, A>() -> Channel<'a, A> {
        Channel {outputs: Vec::new(), marker: PhantomData}
    }
}



pub struct Channel<'a, A> {
    outputs: Vec<(Sender<Event<A>>, &'a Child)>,
    marker: PhantomData<A>,
}

impl<'a, A> Channel<'a, A> {
    fn start(&self) {
        /*
        for (tx, output) in self.outputs.iter() {
            child.start(tx);
        }
        */
    }

    fn emit(&self, a: A) {
    }
}

impl<'a, A> Parent<'a, A> for Channel<'a, A> {
    fn add_output(&mut self, tx: Sender<Event<A>>, child: &'a Child) {
        self.outputs.push((tx, child));
    }
}

impl<'a, A> Signal<'a, A> for Channel<'a, A> {}



pub struct LiftSignal<'a, F, A, B> 
where F: Fn(&A) -> B
{
    f: F,
    rx: Receiver<Event<A>>,
    outputs: Vec<(Sender<Event<B>>, &'a Child)>,
    marker: PhantomData<A>,
}

impl<'a, F, A, B> Parent<'a, B> for LiftSignal<'a, F, A, B>
where F: Fn(&A) -> B
{
    fn add_output(&mut self, tx: Sender<Event<B>>, child: &'a Child) {
        self.outputs.push((tx, child));
    }
}

impl<'a, F, A, B> Signal<'a, B> for LiftSignal<'a, F, A, B> where F: Fn(&A) -> B {}

impl<'a, F, A, B> Child for LiftSignal<'a, F, A, B> 
where F: 'static + Fn(&A) -> B + Clone + Send,
    A: 'static + Send,
    B: 'static + Send + Clone + Eq,
{
    fn start(&self) {
        for &(_, child) in self.outputs.iter() {
            // child.start();
        }

        let f = self.f.clone();
        let outputs = self.outputs.iter().map(|&(ref tx, _)| tx.clone()).collect();
        thread::spawn(move || {
            let runner = LiftRunner::new(f, outputs);
            // runner.run(self.rx);
        });
    }
}



pub struct Lift2Signal<'a, F, A, B, C> 
where F: Fn(&A, &B) -> C
{
    f: F,
    rx_l: Receiver<Event<A>>,
    rx_r: Receiver<Event<B>>,
    outputs: Vec<(Sender<Event<C>>, &'a Child)>,
    marker_a: PhantomData<A>,
    marker_b: PhantomData<B>,
    marker_c: PhantomData<C>,
}

impl<'a, F, A, B, C> Parent<'a, C> for Lift2Signal<'a, F, A, B, C>
where F: Fn(&A, &B) -> C
{
    fn add_output(&mut self, tx: Sender<Event<C>>, child: &'a Child) {
        self.outputs.push((tx, child));
    }
}

impl<'a, F, A, B, C> Signal<'a, C> for Lift2Signal<'a, F, A, B, C> where F: Fn(&A, &B) -> C {}

impl<'a, F, A, B, C> Child for Lift2Signal<'a, F, A, B, C>
where F: 'static + Fn(&A, &B) -> C + Clone + Send,
    A: 'static + Send,
    B: 'static + Send,
    C: 'static + Send + Clone + Eq,
{
    fn start(&self) {
        for &(_, output) in self.outputs.iter() {
            output.start();
        }

        let f = self.f.clone();
        let outputs = self.outputs.iter().map(|&(ref tx, _)| tx.clone()).collect();
        thread::spawn(move || {
            let runner = Lift2Runner::new(f, outputs);
            // runner.run(self.rx_l, self.rx_r);
        });
    }
}



pub struct FoldSignal<'a, F, A, B> 
where F: Fn(&B, &A) -> B
{
    f: F,
    state: B,
    rx: Receiver<Event<A>>,
    outputs: Vec<(Sender<Event<B>>, &'a Child)>,
    marker_a: PhantomData<A>,
    marker_b: PhantomData<B>,
}

impl<'a, F, A, B> Parent<'a, B> for FoldSignal<'a, F, A, B>
where F: Fn(&B, &A) -> B
{
    fn add_output(&mut self, tx: Sender<Event<B>>, child: &'a Child) {
        self.outputs.push((tx, child));
    }
}

impl<'a, F, A, B> Signal<'a, B> for FoldSignal<'a, F, A, B> where F: Fn(&B, &A) -> B {}

impl<'a, F, A, B> Child for FoldSignal<'a, F, A, B>
where F: 'static + Fn(&B, &A) -> B + Clone + Send,
    A: 'static + Send,
    B: 'static + Send + Clone + Eq,
{
    fn start(&self) {
        for &(_, output) in self.outputs.iter() {
            output.start();
        }

        let f = self.f.clone();
        let initial = self.state.clone();
        let outputs = self.outputs.iter().map(|&(ref tx, _)| tx.clone()).collect();
        thread::spawn(move || {
            let runner = FoldRunner::new(f, initial, outputs);
            // runner.run(self.rx);
        });
    }
}
