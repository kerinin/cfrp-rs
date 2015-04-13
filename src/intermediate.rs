use std::thread;
use std::marker::*;
use std::rc::*;
use std::sync::mpsc::*;

use super::runner::{LiftRunner, Lift2Runner, FoldRunner};

pub enum Event<T> {
    Changed(T),
    Same,
}



pub trait Signal<A>: Parent<A> {
    fn lift<'a, F, B>(&self, f: F) -> Rc<LiftSignal<F, A, B>>
    where F: 'static + Fn(&A) -> B + Clone + Send,
        A: 'static + Send,
        B: 'static + Send + Clone + Eq,
    {
        // This is a bit janky
        //
        // We need a channel to send data from upstream to signals to this signal's
        // runner (data_tx, data_rx).  The problem is if we just assign data_rx
        // to self, we can't move it into the spawned thread without taking self
        // along with it.  To make things worse, we need to be able to share self
        // across self's 'parents' (so they can send it data) and the user (so they
        // can define donwstream processes).
        //
        // The solution here is to use an enclosing channel to let us send the 
        // actual data channel to the version of self that's used to spawn the
        // runner.
        //
        let (data_tx, data_rx) = channel();
        let (meta_tx, meta_rx) = channel();
        meta_tx.send(data_rx);

        let (output_tx, output_rx) = channel();
        let signal: Rc<LiftSignal<F, A, B>> = Rc::new(
            LiftSignal {f: f, data_rx: meta_rx, output_tx: output_tx, output_rx: output_rx, marker: PhantomData}
        );
        let sigbox: Box<Child> = Box::new(signal.clone());
        self.add_output(data_tx, sigbox);
        signal
    }

    /*
    fn lift2<F, SB, B, C>(&mut self, b: &mut SB, f: F) -> Lift2Signal<F, A, B, C>
    where F: 'static + Fn(&A, &B) -> C + Clone + Send,
        A: 'static + Send,
        B: 'static + Send,
        SB: Parent<B>,
        C: 'static + Send + Clone + Eq,
    {
        let (tx_l, rx_l) = channel();
        let (tx_r, rx_r) = channel();
        let signal = Lift2Signal {f: f, rx_r: rx_r, rx_l: rx_l, outputs: Vec::new(), marker_a: PhantomData, marker_b: PhantomData, marker_c: PhantomData};
        // self.add_output(tx_l, &signal);
        // b.add_output(tx_r, &signal);
        signal
    }

    fn foldp<F, B>(&mut self, f: F, initial: B) -> FoldSignal<F, A, B>
    where F: 'static + Fn(&B, &A) -> B + Clone + Send,
        A: 'static + Send,
        B: 'static + Send + Clone + Eq,
    {
        let (tx, rx) = channel();
        let signal = FoldSignal {f: f, rx: rx, state: initial, outputs: Vec::new(), marker_a: PhantomData, marker_b: PhantomData};
        // self.add_output(tx, &signal);
        signal
    }
    */
}

// NOTE: Might need custom impls here...
impl<A,T> Signal<A> for Rc<T> where T: Signal<A>, Rc<T>: Parent<A> {}

trait Parent<A> {
    fn add_output(&self, Sender<Event<A>>, Box<Child>);
}

trait Child {
    fn start(&self);
}


pub struct LiftSignal<F, A, B> 
where F: Fn(&A) -> B
{
    f: F,
    data_rx: Receiver<Receiver<Event<A>>>,
    output_tx: Sender<(Sender<Event<B>>, Box<Child>)>,
    output_rx: Receiver<(Sender<Event<B>>, Box<Child>)>,
    marker: PhantomData<A>,
}

impl<F, A, B> Parent<B> for LiftSignal<F, A, B>
where F: Fn(&A) -> B
{
    fn add_output(&self, tx: Sender<Event<B>>, child: Box<Child>) {
        self.output_tx.send((tx, child));
    }
}


impl<F, A, B> Signal<B> for LiftSignal<F, A, B> where F: Fn(&A) -> B {}

impl<F, A, B> Child for Rc<LiftSignal<F, A, B>>
where F: 'static + Fn(&A) -> B + Clone + Send,
    A: 'static + Send,
    B: 'static + Send + Clone + Eq,
{
    fn start(&self) {
        let outputs: Vec<(Sender<Event<B>>, Box<Child>)> = self.output_rx.iter().map(|i| i).collect();
        for &(_, ref child) in outputs.iter() {
            child.start();
        }

        let f = self.f.clone();
        let children = outputs.iter().map(|&(ref tx, _)| tx.clone()).collect();
        match self.data_rx.recv() {
            Ok(rx) => {
                thread::spawn(move || {
                    let mut runner = LiftRunner::new(f, children);
                    runner.run(rx);
                });
            },
            _ => { panic!("Unable to fetch incoming data channel - did you try to run this more than once?") },
        }
    }
}

/*

pub struct Lift2Signal<F, A, B, C> 
where F: Fn(&A, &B) -> C
{
    f: F,
    rx_l: Receiver<Event<A>>,
    rx_r: Receiver<Event<B>>,
    outputs: Vec<(Sender<Event<C>>, &'static Child)>,
    marker_a: PhantomData<A>,
    marker_b: PhantomData<B>,
    marker_c: PhantomData<C>,
}

impl<F, A, B, C> Parent<C> for Lift2Signal<F, A, B, C>
where F: Fn(&A, &B) -> C
{
    fn add_output(&mut self, tx: Sender<Event<C>>, child: &'static Child) {
        self.outputs.push((tx, child));
    }
}

impl<F, A, B, C> Signal<C> for Lift2Signal<F, A, B, C> where F: Fn(&A, &B) -> C {}

impl<F, A, B, C> Child for Lift2Signal<F, A, B, C>
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



pub struct FoldSignal<F, A, B> 
where F: Fn(&B, &A) -> B
{
    f: F,
    state: B,
    rx: Receiver<Event<A>>,
    outputs: Vec<(Sender<Event<B>>, &'static Child)>,
    marker_a: PhantomData<A>,
    marker_b: PhantomData<B>,
}

impl<F, A, B> Parent<B> for FoldSignal<F, A, B>
where F: Fn(&B, &A) -> B
{
    fn add_output(&mut self, tx: Sender<Event<B>>, child: &'static Child) {
        self.outputs.push((tx, child));
    }
}

impl<F, A, B> Signal<B> for FoldSignal<F, A, B> where F: Fn(&B, &A) -> B {}

impl<F, A, B> Child for FoldSignal<F, A, B>
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

pub struct Reactor;

impl Reactor {
    fn channel<A>() -> Channel<A> {
        let (tx, rx) = channel();
        Channel {outputs_tx: tx, outputs_rx: rx, marker: PhantomData}
    }
}



pub struct Channel<A> {
    outputs_tx: Sender<(Sender<Event<A>>, Rc<Box<Child>>)>,
    outputs_rx: Receiver<(Sender<Event<A>>, Rc<Box<Child>>)>,
    marker: PhantomData<A>,
}

impl<A> Channel<A> {
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

impl<A> Parent<A> for Channel<A> {
    fn add_output<T>(&self, tx: Sender<Event<A>>, child: Rc<Box<Child>>)
    {
        self.outputs_tx.send((tx, child));
    }
}

impl<A> Signal<A> for Channel<A> {}
*/

