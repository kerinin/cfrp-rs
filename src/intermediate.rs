use std::thread;
use std::marker::*;
use std::sync::mpsc::*;

use super::runner::{LiftRunner, Lift2Runner, FoldRunner};

pub enum Event<T> {
    Changed(T),
    Same,
}



pub trait Signal<'a, A> {
    fn lift<F, B>(&self, f: F) -> LiftSignal<'a, F, A, B> 
    where F: Fn(&A) -> B,
        B: Clone + Eq,
    {
        LiftSignal {f: f, children: Vec::new(), marker: PhantomData}
    }

    fn lift2<F, SB, B, C>(&self, b: &SB, f: F) -> Lift2Signal<'a, F, A, B, C>
    where F: Fn(&A, &B) -> C,
        C: Clone + Eq,
    {
        Lift2Signal {f: f, children: Vec::new(), marker_a: PhantomData, marker_b: PhantomData, marker_c: PhantomData}
    }

    fn foldp<F, B>(&self, f: F, initial: B) -> FoldSignal<'a, F, A, B>
    where F: Fn(&B, &A) -> B,
        B: Clone + Eq,
    {
        FoldSignal {f: f, state: initial, children: Vec::new(), marker_a: PhantomData, marker_b: PhantomData}
    }
}

trait Child<T> {
    fn start(&self, rx: Receiver<Event<T>>);
}



pub struct Reactor;

impl Reactor {
    fn channel<'a, A>() -> Channel<'a, A> {
        Channel {children: Vec::new(), marker: PhantomData}
    }
}



pub struct Channel<'a, A> {
    children: Vec<&'a Child<A>>,
    marker: PhantomData<A>,
}

impl<'a, A> Channel<'a, A> {
    fn start(&self) {
        for child in self.children.iter() {
            let (tx, rx) = channel();
            child.start(rx);
        }
    }

    fn emit(&self, a: A) {
    }
}
impl<'a, A> Signal<'a, A> for Channel<'a, A> {}



pub struct LiftSignal<'a, F, A, B> 
where F: Fn(&A) -> B
{
    f: F,
    children: Vec<&'a Child<B>>,
    marker: PhantomData<A>,
}

impl<'a, F, A, B> Signal<'a, B> for LiftSignal<'a, F, A, B> where F: Fn(&A) -> B {}

impl<'a, F, A, B> Child<A> for LiftSignal<'a, F, A, B> 
where F: 'static + Fn(&A) -> B + Clone + Send,
    A: 'static + Send,
    B: 'static + Send + Clone + Eq,
{
    fn start(&self, rx: Receiver<Event<A>>) {
        let mut outputs = Vec::new();

        for child in self.children.iter() {
            let (child_tx, child_rx) = channel();
            outputs.push(child_tx);
            child.start(child_rx);
        }

        let f = self.f.clone();
        thread::spawn(move || {
            let mut runner = LiftRunner::new(f, outputs);
            runner.run(rx);
        });
    }
}



pub struct Lift2Signal<'a, F, A, B, C> 
where F: Fn(&A, &B) -> C
{
    f: F,
    children: Vec<&'a Child<C>>,
    marker_a: PhantomData<A>,
    marker_b: PhantomData<B>,
    marker_c: PhantomData<C>,
}

// NOTE: Need to figure out how to initialize with both parents
impl<'a, F, A, B, C> Signal<'a, C> for Lift2Signal<'a, F, A, B, C> where F: Fn(&A, &B) -> C {}

impl<'a, F, A, B, C> Child<A> for Lift2Signal<'a, F, A, B, C>
where F: 'static + Fn(&A, &B) -> C + Clone + Send,
    A: 'static + Send,
    B: 'static + Send,
    C: 'static + Send + Clone + Eq,
{
    fn start(&self, l_rx: Receiver<Event<A>>) {
        let mut outputs = Vec::new();

        for child in self.children.iter() {
            let (child_tx, child_rx) = channel();
            outputs.push(child_tx);
            child.start(child_rx);
        }

        let f = self.f.clone();
        thread::spawn(move || {
            let mut runner = Lift2Runner::new(f, outputs);
            // runner.run(l_rx, r_rx);
        });
    }
}



pub struct FoldSignal<'a, F, A, B> 
where F: Fn(&B, &A) -> B
{
    f: F,
    state: B,
    children: Vec<&'a Child<B>>,
    marker_a: PhantomData<A>,
    marker_b: PhantomData<B>,
}

impl<'a, F, A, B> Signal<'a, B> for FoldSignal<'a, F, A, B> where F: Fn(&B, &A) -> B {}

impl<'a, F, A, B> Child<A> for FoldSignal<'a, F, A, B>
where F: 'static + Fn(&B, &A) -> B + Clone + Send,
    A: 'static + Send,
    B: 'static + Send + Clone + Eq,
{
    fn start(&self, rx: Receiver<Event<A>>) {
        let mut outputs = Vec::new();

        for child in self.children.iter() {
            let (child_tx, child_rx) = channel();
            outputs.push(child_tx);
            child.start(child_rx);
        }

        let f = self.f.clone();
        let initial = self.state.clone();
        thread::spawn(move || {
            let mut runner = FoldRunner::new(f, initial, outputs);
            runner.run(rx);
        });
    }
}
