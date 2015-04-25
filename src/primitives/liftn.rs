use std::sync::mpsc::*;
use std::thread::spawn;

// use super::super::*;
// use super::*

pub trait InputList<Head> {
    type InputPullers: 'static + PullInputs;

    fn run(Head, Self) -> Self::InputPullers;
}

trait PullInputs {
    type Values;

    fn pull(&mut self, any_changed: &mut bool, any_exit: &mut bool) -> Self::Values;
}

struct LiftN<F, A, R, B> where
    F: Fn(<<R as InputList<A>>::InputPullers as PullInputs>::Values) -> B,
    R: InputList<A>,
{
    head: A,
    rest: R,
    f: F,
}

impl<F, A, R, B> Signal<B> for LiftN<F, A, R, B> where
    F: 'static + Send + Fn(<<R as InputList<A>>::InputPullers as PullInputs>::Values) -> B,
    R: InputList<A>,
    A: 'static + Send, R: 'static + Send,
{
    fn push_to(self: Box<Self>, target: Option<Box<Push<B>>>) {
        let inner = *self;
        let LiftN {head, rest, f} = inner;
        let mut input_pullers = InputList::run(head, rest);

        match target {
            Some(mut t) => {
                println!("LiftN::push_to Some");

                loop {
                    let mut any_changed = false;
                    let mut any_exit = false;
                    let values = input_pullers.pull(&mut any_changed, &mut any_exit);

                    match (any_exit, any_changed) {
                        // Propagate exit
                        (true, _) => {
                            println!("LiftN: Received Exit");
                            t.push(Event::Exit)
                        },

                        // Changed data, call the function & pass the return value
                        (false, true) => {
                            println!("LiftN: Changed Data");
                            let b = (f)(values);
                            t.push(Event::Changed(b));
                        },

                        // No changes, proxy it along
                        (false, false) => {
                            println!("LiftN: No Changes");
                            t.push(Event::Unchanged);
                        },
                    }
                }
            },
            None => {
                println!("LiftN::push_to target None");

                loop {
                    let mut any_changed = false;
                    let mut any_exit = false;
                    input_pullers.pull(&mut any_changed, &mut any_exit);
                }
            }
        }
    }
}


impl<H, R0> InputList<Box<Signal<H>>> for (Box<Signal<R0>>,) where
    H: 'static + Send + Clone,
    R0: 'static + Send + Clone,
{
    type InputPullers = (InputPuller<H>, InputPuller<R0>);

    fn run(head: Box<Signal<H>>, rest: Self) -> (InputPuller<H>, InputPuller<R0>) {
        println!("InputList::run for internal signal");
        (input_puller(head), input_puller(rest.0))
    }
}

impl<H, R0> InputList<Box<Signal<H>>> for (Box<Branch<R0>>,) where
    H: 'static + Send + Clone,
    R0: 'static + Send + Clone,
{
    type InputPullers = (InputPuller<H>, InputPuller<R0>);

    fn run(head: Box<Signal<H>>, rest: Self) -> (InputPuller<H>, InputPuller<R0>) {
        println!("InputList::run for branch");
        (input_puller(head), input_puller(rest.0))
    }
}

fn input_puller<T>(upstream: Box<Signal<T>>) -> InputPuller<T> where
    T: 'static + Send,
{
    let (tx, rx) = channel();

    spawn(move || {
        let pusher = InputPusher {
            tx: tx,
        };
        upstream.push_to(Some(Box::new(pusher)));
    });

    InputPuller {
        last: None,
        last_was_no_op: false,
        rx: rx,
    }
}

impl<T0, T1> PullInputs for (InputPuller<T0>, InputPuller<T1>) where
    T0: Clone,
    T1: Clone,
{
    type Values = (Option<T0>, Option<T1>);

    fn pull(&mut self, c: &mut bool, e: &mut bool) -> (Option<T0>, Option<T1>) {
        (self.0.pull(c, e), self.1.pull(c, e))
    }
}



// Passed up the 'push_to' chain, finalizes by sending to a channel
struct InputPusher<A> {
    tx: Sender<Event<A>>,
}

impl<A> Push<A> for InputPusher<A> where
    A: 'static + Send,
{
    fn push(&mut self, event: Event<A>) {
        println!("LiftN::InputPusher::push");

        match self.tx.send(event) {
            _ => {},
        }
    }
}

// Pulls from channel, caches values to populate no-op fields
pub struct InputPuller<A> {
    last: Option<A>,
    last_was_no_op: bool,
    rx: Receiver<Event<A>>,
}

impl<A> InputPuller<A> where
    A: Clone,
{
    fn pull(&mut self, any_changed: &mut bool, any_exit: &mut bool) -> Option<A> {
        // NOTE: There may be a more efficient way of doing this than cloning
        match (self.rx.recv(), self.last.clone(), self.last_was_no_op.clone()) {

            // If the value changed, cache & return it
            (Ok(Event::Changed(a)), _, _) => {
                println!("LiftN::InputPuller::pull handling Event::Changed");

                *any_changed = true;
                self.last = Some(a.clone());
                self.last_was_no_op = false;
                Some(a)
            },

            // If the value didn't change but we have a cached value, return it
            (Ok(Event::Unchanged), Some(a), _) => {
                println!("LiftN::InputPuller::pull handling Event::Unchanged");

                self.last_was_no_op = false;
                Some(a)
            },

            // If the value didn't change and we haven't seen any values in the
            // past, something is very wrong.
            (Ok(Event::Unchanged), None, _) => {
                // Should this really panic?
                panic!("Recevied 'unchanged', but no cached data")
            },

            // If we're just keeping in sync, return None handling this appropriately
            // is the responsiblity of the person using `liftn`
            (Ok(Event::NoOp), _, true) => {
                println!("LiftN::InputPuller::pull handling repeated Event::NoOp");

                None
            },


            (Ok(Event::NoOp), _, false) => {
                println!("LiftN::InputPuller::pull handling first Event::NoOp");

                *any_changed = true;
                self.last_was_no_op = true;
                None
            },

            // Propagate exits
            (Ok(Event::Exit), _, _) => {
                println!("LiftN::InputPuller::pull handling Event::Exit");

                *any_exit = true;
                None
            },

            // Begin exiting if the other end went away
            (Err(e), _, _) => {
                println!("LiftN::InputPuller::pull handling Error {}", e);

                *any_exit = true;
                None
            },
        }
    }
}
