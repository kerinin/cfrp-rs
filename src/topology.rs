use std::cell::*;
use std::sync::*;
use std::sync::mpsc::*;
use std::thread::spawn;
use std::marker::*;

use super::input::{Input, CoordinatedInput, NoOp};
use super::{Signal, Run, Fork, Branch, Channel};

pub struct Builder {
    inputs: RefCell<Vec<Box<CoordinatedInput>>>,
    root_signals: RefCell<Vec<Box<Run>>>,
}

impl Builder {
    pub fn add<A>(&self, root: Box<Signal<A> + Send>) -> Branch<A> where
        A: 'static + Clone + Send,
    {
        let (tx, rx) = channel();
        let fork_txs = Arc::new(Mutex::new(vec![tx]));

        let fork = Fork::new(root, fork_txs.clone());

        self.root_signals.borrow_mut().push(Box::new(fork));

        Branch::new(fork_txs, rx)
    }

    pub fn channel<A>(&self, source_rx: Receiver<A>) -> Channel<A> where
        A: 'static + Clone + Send + Eq,
    {
        let (tx, rx) = channel();
        let input = Input::new(source_rx, tx);

        self.inputs.borrow_mut().push(Box::new(input));

        Channel::new(rx)
    }
}

pub struct Topology<T> {
    builder: Builder,
    marker: PhantomData<T>,
}

impl<T> Topology<T> {
    pub fn build<F>(state: T, f: F) -> Self where 
        F: Fn(&Builder, T),
    {
        let builder = Builder { root_signals: RefCell::new(Vec::new()), inputs: RefCell::new(Vec::new()) };
        f(&builder, state);
        
        Topology { builder: builder, marker: PhantomData }
    }

    pub fn run(self) {
        let Builder {inputs, root_signals} = self.builder;

        for root_signal in root_signals.into_inner().into_iter() {
            spawn(move || {
                root_signal.run();
            });
        }

        let no_ops = Arc::new(Mutex::new(inputs.borrow().iter().map(|i| i.boxed_no_op()).collect::<Vec<Box<NoOp>>>()));
        for (idx, input) in inputs.into_inner().into_iter().enumerate() {
            let no_ops_i = no_ops.clone();
            spawn(move || {
                input.run(idx, no_ops_i);
            });
        }
    }
}
