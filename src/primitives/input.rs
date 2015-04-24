use std::io;
use std::sync::*;
use std::thread::spawn;
use std::sync::mpsc::*;

use super::*;

pub trait RunInput: Send {
    fn run(mut self: Box<Self>, usize, Arc<Mutex<Vec<Box<NoOp>>>>);
    fn boxed_no_op(&self) -> Box<NoOp>;
}

pub trait Input<A> {
    fn pull(&mut self) -> Option<A>;
}

pub trait NoOp: Send {
    fn send_no_change(&self);
    fn send_exit(&self);
}

impl<A> Input<A> for Receiver<A> where
    A: Send,
{
    fn pull(&mut self) -> Option<A> {
        match self.recv() {
            Ok(a) => Some(a),
            Err(_) => None,
        }
    }
}

impl<R> Input<u8> for io::Bytes<R> where
    R: io::Read,
{
    fn pull(&mut self) -> Option<u8> {
        match self.next() {
            Some(Ok(a)) => Some(a),
            _ => None,
        }
    }
}

impl<R> Input<String> for io::Lines<R> where
    R: io::BufRead,
{
    fn pull(&mut self) -> Option<String> {
        match self.next() {
            Some(Ok(a)) => Some(a),
            _ => None,
        }
    }
}

pub struct InternalInput<A> where
    A: 'static + Send + Clone
{
    pub input: Box<Input<A> + Send>,
    pub sink_tx: Sender<Event<A>>,
}

impl<A> RunInput for InternalInput<A> where
    A: 'static + Send + Clone
{
    fn run(mut self: Box<Self>, idx: usize, no_ops: Arc<Mutex<Vec<Box<NoOp>>>>) {
        spawn(move || {
            loop {
                match self.input.pull() {
                    Some(a) => {
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
                    None => {
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
