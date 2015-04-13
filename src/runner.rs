use std::marker::*;
use std::sync::mpsc::*;

use super::intermediate::Event;


pub struct LiftRunner<F, A, B>
where F: Fn(&A) -> B,
{
    f: F,
    cached_b: Option<B>,
    outputs: Vec<Sender<Event<B>>>,
    marker: PhantomData<A>,
}

impl<F, A, B> LiftRunner<F, A, B>
where F: Fn(&A) -> B,
    B: Clone + Eq,
{
    pub fn new(f: F, outputs: Vec<Sender<Event<B>>>) -> Self {
        LiftRunner {
            f: f,
            cached_b: None,
            outputs: outputs,
            marker: PhantomData,
        }
    }

    pub fn run(&mut self, rx: Receiver<Event<A>>) {
        for msg in rx.iter() {
            match msg {
                Event::Changed(a) => {
                    let b = (self.f)(&a);
                    let option_b = Some(b.clone());

                    if option_b != self.cached_b {
                        self.cached_b = option_b;

                        for output in self.outputs.iter() {
                            output.send(Event::Changed(b.clone()));
                        }
                    } else {
                        for output in self.outputs.iter() {
                            output.send(Event::Same);
                        }
                    }
                },
                Event::Same => {
                    for output in self.outputs.iter() {
                        output.send(Event::Same);
                    }
                }
            }
        }
    }
}



pub struct Lift2Runner<F, A, B, C>
where F: Fn(&A, &B) -> C,
{
    f: F,
    cached_a: Option<A>,
    cached_b: Option<B>,
    cached_c: Option<C>,
    outputs: Vec<Sender<Event<C>>>,
}

impl<F, A, B, C> Lift2Runner<F, A, B, C>
where F: Fn(&A, &B) -> C,
    C: Clone + Eq,
{
    pub fn new(f: F, outputs: Vec<Sender<Event<C>>>) -> Self {
        Lift2Runner {
            f: f,
            cached_a: None,
            cached_b: None,
            cached_c: None,
            outputs: outputs,
        }
    }

    pub fn run(&mut self, l_rx: Receiver<Event<A>>, r_rx: Receiver<Event<B>>) {
        loop {
            let msg_l = l_rx.recv();
            let msg_r = r_rx.recv();

            match (msg_l, msg_r) {

                // Both signals have changed.  If f(a,b) has changed, push it
                // to all outputs
                (Ok(Event::Changed(a)), Ok(Event::Changed(b))) => {
                    let c = (self.f)(&a, &b);
                    let option_c = Some(c.clone());

                    if option_c != self.cached_c {
                        self.cached_c = option_c;

                        for output in self.outputs.iter() {
                            output.send(Event::Changed(c.clone()));
                        }
                    } else {
                        for output in self.outputs.iter() {
                            output.send(Event::Same);
                        }
                    }
                },

                // Signal B has changed, use the last-seen value for A to compute
                // f(a,b) and push to outputs if the result has changed
                (Ok(Event::Same), Ok(Event::Changed(b))) => {
                    match self.cached_a {
                        Some(ref a) => {
                            let c = (self.f)(a, &b);
                            let option_c = Some(c.clone());

                            if option_c != self.cached_c {
                                self.cached_c = option_c;

                                for output in self.outputs.iter() {
                                    output.send(Event::Changed(c.clone()));
                                }
                            } else {
                                for output in self.outputs.iter() {
                                    output.send(Event::Same);
                                }
                            }
                        },
                        None => { panic!("Received empty event for A, but no value cached") },
                    }
                },

                // Signal A has changed, use the last-seen value for B to compute
                // f(a,b) and push to outputs if the result has changed
                (Ok(Event::Changed(a)), Ok(Event::Same)) => {
                    match self.cached_b {
                        Some(ref b) => {
                            let c = (self.f)(&a, b);
                            let option_c = Some(c.clone());

                            if option_c != self.cached_c {
                                self.cached_c = option_c;

                                for output in self.outputs.iter() {
                                    output.send(Event::Changed(c.clone()));
                                }
                            } else {
                                for output in self.outputs.iter() {
                                    output.send(Event::Same);
                                }
                            }
                        },
                        None => { panic!("Received empty event for B, but no value cached") },
                    }
                },

                // Neither signal has changed, send unchanged events to outputs
                (Ok(Event::Same), Ok(Event::Same)) => {
                    for output in self.outputs.iter() {
                        output.send(Event::Same);
                    }
                },

                // One of the input channels returned an error, stop running
                // NOTE: panic?
                _ => { return; },
            }
        }
    }
}



pub struct FoldRunner<F, A, B>
where F: Fn(&B, &A) -> B,
{
    f: F,
    state: B,
    outputs: Vec<Sender<Event<B>>>,
    marker: PhantomData<A>,
}

impl<F, A, B> FoldRunner<F, A, B>
where F: Fn(&B, &A) -> B,
    B: Clone + Eq,
{
    pub fn new(f: F, initial: B, outputs: Vec<Sender<Event<B>>>) -> Self {
        FoldRunner {
            f: f,
            state: initial,
            outputs: outputs,
            marker: PhantomData,
        }
    }

    pub fn run(&mut self, rx: Receiver<Event<A>>) {
        for msg in rx.iter() {
            match msg {
                Event::Changed(a) => {
                    let b = (self.f)(&self.state, &a);

                    if b != self.state {
                        self.state = b.clone();

                        for output in self.outputs.iter() {
                            output.send(Event::Changed(b.clone()));
                        }
                    } else {
                        for output in self.outputs.iter() {
                            output.send(Event::Same);
                        }
                    }
                },
                Event::Same => {
                    for output in self.outputs.iter() {
                        output.send(Event::Same);
                    }
                }
            }
        }
    }
}
