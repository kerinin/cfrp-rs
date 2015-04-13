mod intermediate;
mod runner;

/*
use std::thread;
use std::sync::*;
use std::clone::*;
use std::sync::mpsc::*;

pub enum SetupEvent<T> {
    Subscribe(Sender<DataEvent<T>>),
    SetupComplete,
}

pub enum DataEvent<T> {
    Changed(T),
    Same,
}

pub trait Signal<A, B> {

    fn subscribe(&mut self, Sender<Event<A, B>>);

    fn lift<F>(&mut self, f: F) -> SignalHandle<B>
    where F: 'static + Fn(&A) -> B + Send,
        A: 'static + Send,
        B: 'static + Send,
    {
        let (setup_tx, data_tx) = LiftHandler::spawn(f);

        self.subscribe(setup_tx, data_tx);

        SignalHandle {subscriptions: setup_tx }
    }

    /*
    fn lift2<F, SB, B, C>(&mut self, b: &mut SB, f: F) -> SignalHandle<C>
    where F: 'static + Fn(&A, &B) -> C + Send,
        A: 'static + Send,
        B: 'static + Send,
        SB: Signal<B>,
        C: 'static + Send,
    {
        let (l_tx, l_rx) = channel();
        let (r_tx, r_rx) = channel();

        self.subscribe(l_tx.clone());
        b.subscribe(r_tx.clone());

        Lift2Handler::spawn(l_rx, r_rx, f);
        SignalHandle {subscriptions: l_tx }
    }

    fn foldp<F, B>(&mut self, f: F, initial: B) -> SignalHandle<B>
    where F: 'static + Fn(&A, B) -> B + Send,
        A: 'static + Send,
        B: 'static + Send,
    {
        let (tx, rx) = channel();
        self.subscribe(tx.clone());

        FoldHandler::spawn(rx, f, initial);
        SignalHandle {subscriptions: tx}
    }
    */
}

/*
pub struct Channel<A> {
    clock: usize,
    subscriptions: Vec<Sender<Event<A>>>,
}

impl<A> Channel<A> {
    pub fn new() -> Channel<A> {
        Channel {clock: 0, subscriptions: Vec::new()}
    }

    pub fn emit(&mut self, a: A) where A: Clone {
        for subscriber in self.subscriptions.iter() {
            subscriber.send(Event::Value(self.clock.clone(), a.clone()));
        }
        self.clock += 1;
    }
}

impl<A> Signal<A> for Channel<A> {
    fn subscribe(&mut self, tx: Sender<Event<A>>) {
        self.subscriptions.push(tx);
    }
}
*/

pub struct SignalHandle<A> {
    subscriptions: Sender<Setup<A>>,
}

impl<A> Signal<A> for SignalHandle<A> {
    fn subscribe(&mut self, event_tx: Sender<Event<A>>) {
        self.subscriptions.send(Setup::Subscribe(tx));
    }
}

struct LiftHandler<F, A, B>
{
    f: F,
    event_rx: Receiver<Event<A>>,
    subscriptions: Vec<(Sender<SetupEvent<B>>, Sender<DataEvent<B>)>>,
}

impl<F, A, B> LiftHandler<F, A, B> {
    fn spawn(f: F) -> (Sender<SetupEvent<B>>, Sender<DataEvent<B>>)
        where F: 'static + Fn(&A) -> B + Send,
            A: 'static + Send,
            B: 'static + Send,
    {
        let (setup_tx, setup_rx) = channel();
        let (data_tx, data_rx) = channel();

        thread::spawn(move || {
            let signal = LiftHandler {
                f: f, 
                setup_rx: setup_rx,
                data_rx: data_rx,
                subscriptions: Vec::new(),
            };
        });

        (setup_tx, setup_rx)
    }

    fn run(&mut self) where A: 'static + Send, B: 'static + Send
    {
    }
}
/*
struct FoldHandler<F, A, B>
{
    f: F,
    state: B,
    rx: Receiver<Event<A>>,
    subscriptions: Vec<Sender<Event<B>>>,
}

impl<F, A, B> FoldHandler<F, A, B> {
    fn spawn(rx: Receiver<Event<A>>, f: F, initial: B)
        where F: 'static + Fn(&A, B) -> B + Send,
            A: 'static + Send,
            B: 'static + Send,
    {
        thread::spawn(move || {
            let signal = FoldHandler {
                f: f, 
                state: initial,
                rx: rx.clone(), 
                subscriptions: Vec::new(),
            };
        });
    }
}

struct Lift2Handler<F, A, B, C>
{
    f: F,
    l_data_rx: Receiver<Event<A>>,
    r_data_rx: Receiver<Event<B>>,
    subscriptions: Vec<Sender<Event<C>>>,
}

impl<F, A, B, C> Lift2Handler<F, A, B, C> {
    fn spawn(l_rx: Receiver<Event<A>>, r_rx: Receiver<Event<B>>, f: F)
        where F: 'static + Fn(&A, &B) -> C + Send,
            A: 'static + Send,
            B: 'static + Send,
            C: 'static + Send,
    {
        thread::spawn(move || {
            let signal = Lift2Handler {
                f: f, 
                l_rx: l_rx.clone(), 
                r_rx: r_rx, 
                subscriptions: Vec::new(),
            };
        });
    }
}
*/

#[test]
fn it_works() {
    let mut ch = signal::Channel::new();
    ch.lift(|i| { -i });
    ch.emit(1);
}
*/
