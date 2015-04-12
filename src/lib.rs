use std::clone::*;

use std::sync::mpsc::*;

enum Memoized<A> {
    Changed(usize, A),
    Unchanged(usize),
}

pub trait Signal<A, B, F> 
where F: Fn(A) -> B
{
    fn lift(&mut self, f: F) -> LiftedSignal<A, B, F>;
}

pub struct Channel<A> {
    id: usize,
    memo: Option<A>,
    sinks: Vec<Sender<Memoized<A>>>,
}

pub struct LiftedSignal<A, B, F>
where F: Fn(A) -> B
{
    f: F,
    memo: Option<A>,
    source: Receiver<Memoized<A>>,
    sinks: Vec<Sender<Memoized<B>>>,
}

impl<A, B, F> LiftedSignal<A, B, F> 
where A: Clone + Eq, B: Clone, F: Fn(A) -> B
{
    fn subscribe(&mut self) -> Receiver<Memoized<B>> {
        let (tx, rx) = channel();
        self.sinks.push(tx);
        return rx;
    }

    fn push_unchanged(&self, id: usize) {
        for sink in self.sinks.iter() {
            sink.send(Memoized::Unchanged(id.clone()));
        }
    }

    fn push_changed(&mut self, id: usize, a: A) {
        self.memo = Some(a.clone());
        let b = self.f.call_once((a.clone(),));

        for sink in self.sinks.iter() {
            sink.send(Memoized::Changed(id.clone(), b.clone()));
        }
    }

    fn run(&mut self) {
        loop {
            let r = self.source.recv();

            match r {
                Ok(msg) => {
                    match msg {
                        Memoized::Changed(id, a) => self.push_changed(id, a),
                        Memoized::Unchanged(id) => self.push_unchanged(id),
                    }
                },
                Err(_) => {},
            }
        }
    }
}

/*
impl<A, B, F> Signal<A, B, F> for LiftedSignal<A, B, F> 
where A: Clone + Eq, B: Clone, F: Fn(A) -> B
{
    fn lift<C>(&mut self, f: F) -> LiftedSignal<B, C> {
        let rx = self.subscribe();

        LiftedSignal {f: f, memo: None, source: rx, sinks: Vec::new()}
    }
}

impl<A, B, F> Channel<A> 
where A: Clone + Eq, F: Fn(A) -> B
{
    pub fn send(&mut self, a: &A) {
        self.id += 1;

        match self.memo {
            Some(ref old_a) => {
                if old_a == a {
                    for sink in self.sinks.iter() {
                        sink.send(Memoized::Unchanged(self.id.clone()));
                    }
                } else {
                    for sink in self.sinks.iter() {
                        sink.send(Memoized::Changed(self.id.clone(), a.clone()));
                    }
                }
            },
            _ => {
                for sink in self.sinks.iter() {
                    sink.send(Memoized::Changed(self.id.clone(), a.clone()));
                }
            }
        }
    }

    pub fn lift<B>(&mut self, f: F) -> LiftedSignal<A, B> {
        let rx = self.subscribe();

        LiftedSignal {f: f, memo: None, source: rx, sinks: Vec::new()}
    }

    fn subscribe(&mut self) -> Receiver<Memoized<A>> {
        let (tx, rx) = channel();
        self.sinks.push(tx);
        return rx;
    }
}
*/

#[test]
fn it_works() {
    let mut r: Reactor = Default::default();

    let (input_channel, input_signal) = r.channel::<isize>();
    let negated = r.lift(|i| { -i }, input_signal);
    let accumulated = r.foldp(|a, i| { a + i }, 0, negated);
    let printed = r.lift(|i| { println!("{}", i); }, accumulated);

    input_channel.send(0);
    input_channel.send(1);
    input_channel.send(-1);
}
