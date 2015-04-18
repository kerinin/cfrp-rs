use std::thread::*;
use std::clone::*;
use std::cell::*;
use std::sync::*;
use std::sync::mpsc::*;

use super::*;
use super::lift::Lift;

trait RunInput: Send {
    fn no_op(&self) -> Box<NoOp>;
    fn run(self: Box<Self>, idx: usize, no_ops: Arc<Mutex<Vec<Box<NoOp>>>>);
}

pub struct Input<A> {
    source_rx: Receiver<A>,
    sink_tx: Sender<Option<A>>,
}

pub struct Coordinator {
    inputs: RefCell<Vec<Box<RunInput>>>,
}

impl Coordinator {
    pub fn channel<'a, A>(&'a self, source_rx: Receiver<A>) -> Lift<fn(&A) -> A, A, A> where
        A: 'static + Send + Clone,
    {
        let (sink_tx, sink_rx) = channel();

        self.inputs.borrow_mut().push(
            Box::new(
                Input {
                    source_rx: source_rx,
                    sink_tx: sink_tx,
                }
            )
        );

        Lift::new(Box::new(Clone::clone), sink_rx)
    }
}

impl Run for Coordinator {
    fn run(mut self) {
        let no_ops: Arc<Mutex<Vec<Box<NoOp>>>> = Arc::new(
            Mutex::new(
                self.inputs.borrow().iter().map(|input| input.no_op()).collect()
            )
        );

        for (i, input) in self.inputs.into_inner().into_iter().enumerate() {
            let thread_no_ops = no_ops.clone();
            spawn(move || {
                input.run(i, thread_no_ops);
            });
        }
    }
}

impl<A> RunInput for Input<A> where
    A: 'static + Send + Clone,
{
    fn no_op(&self) -> Box<NoOp> {
        Box::new(self.sink_tx.clone())
    }

    fn run(self: Box<Self>, idx: usize, no_ops: Arc<Mutex<Vec<Box<NoOp>>>>) {
        loop {
            match self.source_rx.recv() {
                Ok(ref a) => {
                    // NOTE: Memoize!
                    for (i, no_op) in no_ops.lock().unwrap().iter().enumerate() {
                        if i == idx {
                            self.sink_tx.send(Some(a.clone()));
                        } else {
                            no_op.no_op();
                        }
                    }
                }
                _ => { return }
            }
        }
    }
}

