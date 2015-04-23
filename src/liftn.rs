use std::sync::mpsc::*;
use std::thread::spawn;

use super::{InternalSignal, Event, Push};


struct LiftN<F, A, R, B> where
    F: Fn(<<R as InputList<A>>::InputPullers as PullInputs>::Values) -> B,
    R: InputList<A>,
{
    head: A,
    rest: R,
    f: F,
}

impl<F, A, R, B> InternalSignal<B> for LiftN<F, A, R, B> where
    F: 'static + Send + Fn(<<R as InputList<A>>::InputPullers as PullInputs>::Values) -> B,
    R: InputList<A>,
    A: 'static + Send, R: 'static + Send,
{
    fn push_to(self: Box<Self>, target: Option<Box<Push<B>>>) {
        let inner = *self;
        let LiftN {head, rest, f} = inner;
        let input_pullers = InputList::run(head, rest);

        match target {
            Some(mut t) => {
                loop {
                    let mut any_changed = false;
                    let mut any_exit = false;
                    let values = input_pullers.pull(&mut any_changed, &mut any_exit);
                    let b = (f)(values);
                    t.push(Event::Changed(b));
                }
            },
            None => {
                loop {
                    let mut any_changed = false;
                    let mut any_exit = false;
                    input_pullers.pull(&mut any_changed, &mut any_exit);
                }
            }
        }
    }
}






trait InputList<Head> {
    type InputPullers: PullInputs;

    fn run(Head, Self) -> Self::InputPullers;
}

impl<H, R0> InputList<Box<InternalSignal<H>>> for (Box<InternalSignal<R0>>,) where
    H: 'static + Send,
    R0: 'static + Send,
{
    type InputPullers = (InputPuller<H>, InputPuller<R0>);

    fn run(head: Box<InternalSignal<H>>, rest: Self) -> (InputPuller<H>, InputPuller<R0>) {
        (input_puller(head), input_puller(rest.0))
    }
}

fn input_puller<T>(upstream: Box<InternalSignal<T>>) -> InputPuller<T> where
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
        rx: rx,
    }
}

trait PullInputs {
    type Values;

    fn pull(&self, any_changed: &mut bool, any_exit: &mut bool) -> Self::Values;
}

impl<T0, T1> PullInputs for (InputPuller<T0>, InputPuller<T1>) {
    type Values = (Option<T0>, Option<T1>);

    fn pull(&self, any_changed: &mut bool, any_exit: &mut bool) -> (Option<T0>, Option<T1>) {
        (self.0.pull(), self.1.pull())
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
    }
}

// Pulls from channel, caches values to populate no-op fields
struct InputPuller<A> {
    rx: Receiver<Event<A>>,
}

impl<A> InputPuller<A> {
    fn pull(&self) -> Option<A> {
        None
    }
}

/*
// Combines the pulled data for all inputs and pushes downstream
struct OutputPusher<A> {
}

impl<A> Push<A> for LiftNPusher<A> where
    A: 'static + Send,
{
    fn push(&mut self, event: Event<A>) {
        match self.tx.send(event) {
            _ => {},
        }
    }
}

impl<A> InputPuller<A> {
    fn pull(&mut self, any_changed: &mut bool, any_exit: &mut bool) -> Option<A> {
    }
}

impl<A> OutputPusher<A> {
    fn push(&mut self, event: Event<A>) {
    }
}












fn push<T>(signal: Box<InternalSignal<T>>) -> Receiver<Event<T>> where
    T: 'static + Send,
{
    let (tx, rx) = channel();

    spawn(move || {
        let pusher = LiftNPusher {
            tx: tx,
        };

        signal.push_to(Some(Box::new(pusher)));
    });

    LiftNPuller {
        rx: rx
    }
}


// Spawn threads to push data from various lengths of tuples

trait RunBranches<Head> {
    type Receivers;

    fn run(Head, Self) -> Self::Receivers;
}

impl<H, R0> RunBranches<Box<InternalSignal<H>>> for (Box<InternalSignal<R0>>,) where
    H: 'static + Send,
    R0: 'static + Send,
{
    type Receivers = (LiftNPuller<H>, LiftNPuller<R0>);

    fn run(head: Box<InternalSignal<H>>, rest: Self) -> (LiftNPuller<H>, LiftNPuller<R0>) {
        (push(head), push(rest.0))
    }
}


// Pull upstream data from channels

fn pull<T>(puller: LiftNPuller<T>) -> T {
    match puller.rx.recv() {
        Ok(e) => e,
        Err(_) => Event::Exit,
    }
}

trait PullValues {
    type Values;

    fn pull_values(self) -> Self::Values;
}

impl<V0, V1> PullValues for (Receiver<Event<V0>>, Receiver<Event<V1>>)
{
    // NOTE: This should pull VALUES, not events
    type Values = (Event<V0>, Event<V1>);

    fn pull_values(self) -> (Event<V0>, Event<V1>) {
        (pull(self.0), pull(self.1)) 
    }
}

// PREVIOUS CODE
struct LiftNPusher<A> {
    tx: Sender<Event<A>>,
}

impl<A> Push<A> for LiftNPusher<A> where
    A: 'static + Send,
{
    fn push(&mut self, event: Event<A>) {
        match self.tx.send(event) {
            _ => {},
        }
    }
}

fn push<T>(signal: Box<InternalSignal<T>>) -> Receiver<Event<T>> where
    T: 'static + Send,
    InternalSignal<T>: Send,
{
    let (tx, rx) = channel();

    spawn(move || {
        let pusher = LiftNPusher {
            tx: tx,
        };

        signal.push_to(Some(Box::new(pusher)));
    });

    rx
}

fn pull<T>(rx: Receiver<Event<T>>) -> Event<T> {
    match rx.recv() {
        Ok(e) => e,
        Err(_) => Event::Exit,
    }
}

impl<A, B, C> Rest<Box<A>> for (Box<B>, Box<C>) where
    A: 'static + Send + InternalSignal<A>,
    B: 'static + Send + InternalSignal<B>,
    C: 'static + Send + InternalSignal<C>,
    InternalSignal<A>: Send,
    InternalSignal<B>: Send,
    InternalSignal<C>: Send,
{
    type List = (Box<A>, Box<B>, Box<C>);
    type Receivers = (Receiver<Event<A>>, Receiver<Event<B>>, Receiver<Event<C>>);
    type Values = (A, B, C);

    fn cons(self, head: Box<A>) -> <Self as Rest<Box<A>>>::List {
        (head, self.0, self.1)
    }

    fn run(list: <Self as Rest<Box<A>>>::List) -> <Self as Rest<Box<A>>>::Receivers {
        (push(list.0), push(list.1), push(list.2))
    }

    fn pull(list: <Self as Rest<Box<A>>>::Receivers) -> <Self as Rest<Box<A>>>::Values {
        (pull(list.0), pull(list.1), pull(list.2))
    }
}

struct LiftN<F, A, R, B> where
    F: Fn(<R as Rest<A>>::Values) -> B,
    R: Rest<A>,
    A: 'static + Send,
    R: 'static + Send,
    B: 'static + Send,
{
    upstream: <R as Rest<A>>::List,
    f: F,
}

impl<F, A, R, B> InternalSignal<B> for LiftN<F, A, R, B> where
    F: Fn(<R as Rest<A>>::Values) -> B,
    R: Rest<A>,
    A: 'static + Send,
    R: 'static + Send,
    B: 'static + Send,
{
    fn push_to(self: Box<Self>, target: Option<Box<Push<B>>>) {
        // let receivers: <R as Rest<A>>::Values = Rest::<A>::run(self.upstream);
        // let receivers: <R as Rest<A>>::Receivers = Rest::run(self.upstream);
        self.upstream

        match target {
            Some(t) => {
                loop {
                    let values = Rest::pull(receivers);
                    let b = (self.f)(values);
                    t.push(Event::Changed(b));
                }
            },
            None => {
                loop {
                    Rest::pull(receivers);
                }
            }
        }
    }
}
*/
