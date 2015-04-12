use std::sync::mpsc::channel;

enum Memoized<T> {
    Changed(usize, T),
    Unchanged(usize),
}

pub struct LiftedSignal<A, B> {
    f: Fn(A) -> B,
    memo: Option<A>,
    source: Receiver<Memoized<A>>,
    sinks: Vec<Sender<Memoized<B>>>,
}

impl<A, B> LiftedSignal<A, B> {
    fn lift<C>(&mut self, f: Fn(B) -> C) -> LiftedSignal<B, C> {
        let rx = self.subscribe();

        LiftedSignal {f: f, memo: None, source: rx, sinks: Vec::new()}
    }

    fn subscribe(&mut self) -> Receiver<Memoized<B>> {
        let (tx, rx) = channel();
        self.sinks.push(tx);
        return rx;
    }

    fn run(&self) {
        for msg in self.source.iter() {
            match msg {
                Unchanged(id) => {
                    for sink in self.sinks.iter() {
                        sink.send(Unchanged(id.clone()));
                    }
                },
                Changed(id, a) => {
                    if Some(a) == self.memo {
                        for sink in self.sinks.iter() {
                            sink.send(Unchanged(id.clone()));
                        }
                    } else {
                        let b = self.f(a);
                        self.memo = Some(b);

                        for sink in self.sinks.iter() {
                            sink.send(Changed(id.clone(), b.clone()));
                        }
                    }
                }
            }
        }
    }
}
