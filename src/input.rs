use std::sync::*;
use std::sync::mpsc::*;

pub trait NoOp: Send {
    fn send_no_change(&self);
}

pub trait CoordinatedInput: Send {
    fn run(self: Box<Self>, usize, Arc<Mutex<Vec<Box<NoOp>>>>);
    fn boxed_no_op(&self) -> Box<NoOp>;
}

pub struct Input<A> where
    A: 'static + Send
{
    source_rx: Receiver<A>,
    sink_tx: Sender<Option<A>>,
    last: Option<A>,
}

impl<A> Input<A> where
    A: 'static + Send
{
    pub fn new(source_rx: Receiver<A>, sink_tx: Sender<Option<A>>) -> Input<A> {
        Input {
            source_rx: source_rx,
            sink_tx: sink_tx,
            last: None,
        }
    }
}

impl<A> CoordinatedInput for Input<A> where
    A: 'static + Clone + Send + Eq,
{
    fn run(mut self: Box<Self>, idx: usize, no_ops: Arc<Mutex<Vec<Box<NoOp>>>>) {
        loop {
            match self.source_rx.recv() {
                Ok(ref a) => {
                    if self.last.as_ref() != Some(a) {
                        for (i, ref no_op) in no_ops.lock().unwrap().iter().enumerate() {
                            if i == idx {
                                self.sink_tx.send(Some(a.clone()));
                            } else {
                                no_op.send_no_change();
                            }
                        }
                        self.last = Some(a.clone());
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
