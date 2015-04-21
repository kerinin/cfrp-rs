use std::sync::*;
use std::thread::spawn;
use std::sync::mpsc::*;

use super::Event;

pub trait NoOp: Send {
    fn send_no_change(&self);
    fn send_exit(&self);
}

pub trait CoordinatedInput: Send {
    fn run(self: Box<Self>, usize, Arc<Mutex<Vec<Box<NoOp>>>>);
    fn boxed_no_op(&self) -> Box<NoOp>;
}

pub struct Input<A> where
    A: 'static + Send + Clone
{
    source_rx: Receiver<A>,
    sink_tx: Sender<Event<A>>,
}

impl<A> Input<A> where
    A: 'static + Send + Clone
{
    pub fn new(source_rx: Receiver<A>, sink_tx: Sender<Event<A>>) -> Input<A> {
        Input {
            source_rx: source_rx,
            sink_tx: sink_tx,
        }
    }
}

impl<A> CoordinatedInput for Input<A> where
    A: 'static + Send + Clone
{
    fn run(self: Box<Self>, idx: usize, no_ops: Arc<Mutex<Vec<Box<NoOp>>>>) {
        spawn(move || {
            loop {
                match self.source_rx.recv() {
                    Ok(a) => {
                        let received = Event::Changed(a);

                        for (i, ref no_op) in no_ops.lock().unwrap().iter().enumerate() {
                            if i == idx {
                                match self.sink_tx.send(received.clone()) {
                                    // We can't really terminate a child process, so just ignore errors...
                                    _ => {}
                                }
                            } else {
                                no_op.send_no_change();
                            }
                        }
                    },
                    Err(_) => {
                        // NOTE: Can we be less drastic here?
                        for (_, ref no_op) in no_ops.lock().unwrap().iter().enumerate() {
                            no_op.send_exit()
                        }

                        return
                    },
                }
            }
        });
    }

    fn boxed_no_op(&self) -> Box<NoOp> {
        Box::new(self.sink_tx.clone())
    }
}

impl<A> NoOp for Sender<Event<A>> where
    A: 'static + Send,
{
    fn send_no_change(&self) {
        match self.send(Event::NoOp) {
            // We can't really terminate a child process, so just ignore errors...
            _ => {}
        }
    }

    fn send_exit(&self) {
        match self.send(Event::Exit) {
            // We can't really terminate a child process, so just ignore errors...
            _ => {}
        }
    }
}
