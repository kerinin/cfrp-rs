use std::clone::*;
use std::sync::mpsc::*;

// ???
pub trait Channel<A> {
}

pub trait Signal<A> {
    fn lift<F, B>(&mut self, f: F) -> LiftedSignal<F, A, B> where F: Fn(&A) -> B;
    fn foldp<F, B>(&mut self, f: F, b: B) -> FoldedSignal<F, A, B> where F: Fn(&A, B) -> B;
}

enum Memoized<A> {
    Changed(usize, A),
    Unchanged(usize),
}

pub struct LiftedSignal<F, A, B>
{
    f: F,
    memo: Option<A>,
    source: Receiver<Memoized<A>>,
    sinks: Vec<Sender<Memoized<B>>>,
}

pub struct FoldedSignal<F, A, B>
{
    f: F,
    memo: Option<A>,
    state: B,
    source: Receiver<Memoized<A>>,
    sinks: Vec<Sender<Memoized<B>>>,
}

impl<F, A, B> Signal<B> for LiftedSignal<F, A, B> 
{
    fn lift<G, C>(&mut self, g: G) -> LiftedSignal<G, B, C> where G: Fn(&B) -> C
    {
        let rx = self.subscribe();
        LiftedSignal {f: g, memo: None, source: rx, sinks: Vec::new()}
    }

    fn foldp<G, C>(&mut self, g: G, c: C) -> FoldedSignal<G, B, C> where G: Fn(&B, C) -> C
    {
        let rx = self.subscribe();
        FoldedSignal {f: g, memo: None, state: c, source: rx, sinks: Vec::new()}
    }
}

impl<F, A, B> Signal<B> for FoldedSignal<F, A, B> 
{
    fn lift<G, C>(&mut self, g: G) -> LiftedSignal<G, B, C> where G: Fn(&B) -> C
    {
        let rx = self.subscribe();
        LiftedSignal {f: g, memo: None, source: rx, sinks: Vec::new()}
    }

    fn foldp<G, C>(&mut self, g: G, c: C) -> FoldedSignal<G, B, C> where G: Fn(&B, C) -> C
    {
        let rx = self.subscribe();
        FoldedSignal {f: g, memo: None, state: c, source: rx, sinks: Vec::new()}
    }
}

impl<F, A, B> LiftedSignal<F, A, B> 
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
}

impl<F, A, B> FoldedSignal<F, A, B> 
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
}

impl<F, A, B> LiftedSignal<F, A, B> 
where A: Clone + Eq, B: Clone, F: Fn(&A) -> B
{
    fn push_changed(&mut self, id: usize, a: &A) {
        self.memo = Some(a.clone());
        let b = (self.f)(a);

        for sink in self.sinks.iter() {
            sink.send(Memoized::Changed(id.clone(), b.clone()));
        }
    }

    fn run(&mut self) {
        loop {
            let r = self.source.recv();
            let memo = self.memo.clone(); // Workaround becasue need to borrow &self mutably

            match r {
                Ok(msg) => {
                    match (msg, memo) {
                        (Memoized::Changed(id, ref a), None) => self.push_changed(id, a),
                        (Memoized::Changed(id, ref a), Some(ref old_a)) if a != old_a => self.push_changed(id, a),
                        (Memoized::Changed(id, _), _) | (Memoized::Unchanged(id), _) => self.push_unchanged(id),
                    }
                },
                Err(_) => {},
            }
        }
    }
}

impl<F, A, B> FoldedSignal<F, A, B> 
where A: Clone + Eq, B: Clone, F: Fn(&A, B) -> B
{
    fn push_changed(&mut self, id: usize, a: &A) {
        self.memo = Some(a.clone());
        self.state = (self.f)(a, self.state.clone());

        for sink in self.sinks.iter() {
            sink.send(Memoized::Changed(id.clone(), self.state.clone()));
        }
    }

    fn run(&mut self) {
        loop {
            let r = self.source.recv();
            let memo = self.memo.clone(); // Workaround becasue need to borrow &self mutably

            match r {
                Ok(msg) => {
                    match (msg, memo) {
                        (Memoized::Changed(id, ref a), None) => self.push_changed(id, a),
                        (Memoized::Changed(id, ref a), Some(ref old_a)) if a != old_a => self.push_changed(id, a),
                        (Memoized::Changed(id, _), _) | (Memoized::Unchanged(id), _) => self.push_unchanged(id),
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
*/
