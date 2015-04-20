use std::marker::*;
use std::sync::*;
use std::cell::*;
use std::thread::spawn;
use std::sync::mpsc::*;

#[derive(Debug, PartialEq, Eq)]
pub enum Event<A> where
    A: 'static + Send,
{
    Changed(A),
    NoChange,
    Exit,
}

pub trait Signal<A>
{
    fn recv(&self) -> Option<A>;
}

pub trait Run: Send {
    fn run(self: Box<Self>);
}

trait NoOp: Send {
    fn send_no_change(&self);
}

trait CoordinatedInput: Send {
    fn run(self: Box<Self>, usize, Arc<Mutex<Vec<Box<NoOp>>>>);
    fn boxed_no_op(&self) -> Box<NoOp>;
}

struct Input<A> where
    A: 'static + Send
{
    source_rx: Receiver<A>,
    sink_tx: Sender<Option<A>>,
}

impl<A> CoordinatedInput for Input<A> where
    A: 'static + Clone + Send,
{
    fn run(self: Box<Self>, idx: usize, no_ops: Arc<Mutex<Vec<Box<NoOp>>>>) {
        loop {
            match self.source_rx.recv() {
                Ok(ref a) => {
                    for (i, ref no_op) in no_ops.lock().unwrap().iter().enumerate() {
                        if i == idx {
                            self.sink_tx.send(Some(a.clone()));
                        } else {
                            no_op.send_no_change();
                        }
                    }
                },
                Err(_) => {
                    for (i, no_op) in no_ops.lock().unwrap().iter().enumerate() {
                        if i == idx {
                            self.sink_tx.send(None);
                        } else {
                            no_op.send_no_change();
                        }
                    }
                    return;
                },
            }
        }
    }

    fn boxed_no_op(&self) -> Box<NoOp> {
        Box::new(self.sink_tx.clone())
    }
}

impl<A> NoOp for Sender<Option<A>> where
    A: 'static + Send,
{
    fn send_no_change(&self) {
        self.send(None);
    }
}

pub struct Channel<A> where
    A: 'static + Send,
{
    source_rx: Receiver<Option<A>>,
}

impl<A> Signal<A> for Channel<A> where
    A: 'static + Send,
{
    fn recv(&self) -> Option<A> {
        match self.source_rx.recv() {
            Err(_) => None,
            Ok(a) => { println!("Received something"); a },
        }
    }
}

impl<A> Channel<A> where
    A: 'static + Send,
{
    pub fn lift<F, B>(self, f: F) -> Lift<F, A, B> where
        F: 'static + Send + Fn(&A) -> B,
        B: 'static + Send,
    {
        Lift {
            parent: Box::new(self),
            f: f,
        }
    }
}

pub struct Lift<F, A, B> where
    F: 'static + Send + Fn(&A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{
    parent: Box<Signal<A> + Send>,
    f: F,
}

impl<F, A, B> Signal<B> for Lift<F, A, B> where
    F: 'static + Send + Fn(&A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{
    fn recv(&self) -> Option<B> {
       match self.parent.recv() {
           Some(ref a) => { println!("Received something"); Some((self.f)(a)) },
           None => None,
       }
    }
}

impl<F, A, B> Run for Lift<F, A, B> where
    F: 'static + Send + Fn(&A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{
    fn run(self: Box<Self>) {
        loop {
            self.recv();
        }
    }
}

pub struct Fork<A> where
    A: 'static + Send,
{
    parent: Box<Signal<A> + Send>,
    sink_txs: Arc<Mutex<Vec<Sender<Option<A>>>>>,
}

impl<A> Run for Fork<A> where
    A: 'static + Clone + Send,
{
    fn run(self: Box<Self>) {
        loop {
            match self.parent.recv() {
                Some(a) => {
                    for sink in self.sink_txs.lock().unwrap().iter() {
                        sink.send(Some(a.clone()));
                    }
                },
                _ => {},
            }
        }
    }
}

pub struct Branch<A> where
    A: 'static + Send,
{
    // Arc<T> is send if T: Send + Sync (which mutex is, unconditionally)
    fork_txs: Arc<Mutex<Vec<Sender<Option<A>>>>>,
    source_rx: Receiver<Option<A>>,
}

impl<A> Clone for Branch<A> where
    A: 'static + Send,
{
    fn clone(&self) -> Branch<A> {
        let (tx, rx) = channel();
        self.fork_txs.lock().unwrap().push(tx);
        Branch { fork_txs: self.fork_txs.clone(), source_rx: rx }
    }
}

impl<A> Signal<A> for Branch<A> where
    A: 'static + Send,
{
    fn recv(&self) -> Option<A> {
        match self.source_rx.recv() {
            Err(_) => None,
            Ok(a) => { println!("Received something"); a },
        }
    }
}


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

        let fork = Fork {
            parent: root,
            sink_txs: fork_txs.clone(),
        };

        self.root_signals.borrow_mut().push(Box::new(fork));

        Branch { fork_txs: fork_txs, source_rx: rx }
    }

    pub fn channel<A>(&self, source_rx: Receiver<A>) -> Channel<A> where
        A: 'static + Clone + Send,
    {
        let (tx, rx) = channel();
        let input = Input {
            source_rx: source_rx,
            sink_tx: tx,
        };

        self.inputs.borrow_mut().push(Box::new(input));

        Channel {
            source_rx: rx,
        }
    }
}

pub struct Topology<T> {
    builder: Builder,
    marker: PhantomData<T>,
}

impl<T> Topology<T> {
    pub fn build<F>(state: T, mut f: F) -> Self where 
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


#[cfg(test)] 
mod test {
    // extern crate quickcheck;
    use std::sync::mpsc::*;
    
    use super::*;

    #[test]
    fn integration() {
        let (in_tx, in_rx) = channel();
        let (out_tx, out_rx): (Sender<Option<usize>>, Receiver<Option<usize>>) = channel();

        Topology::build( (in_rx, out_tx), |t, (in_rx, out_tx)| {

            let channel: Channel<usize> = t.channel(in_rx);
            let lift = channel.lift(|i: &usize| -> usize { i + 1 });
            let plus_one = t.add(Box::new(lift));

            // t.add(Box::new(
            //     plus_one.
            //         lift(|i: &usize| -> usize { i + 1 })
            // ));
        }).run();

        in_tx.send(0);
        // NOTE: Not actually delivering data to out_rx ATM
        // Also, need to do a bunch of optimizations around memoization and such
        assert_eq!(Some(1), out_rx.recv().unwrap())
    }
}
