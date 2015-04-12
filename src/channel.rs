use std::sync::mpsc::Sender;

pub struct Channel<T> {
    id: usize,
    memo: Option<T>,
    sinks: Vec<Sender<Memoized<T>>>,
}

impl<A> Channel<A> {
    pub fn send(&self, a: A) {
        self.id += 1;

        if Some(a) == self.memo {
            for sink in self.sinks.iter() {
                sink.send(Unchanged(self.id.clone()));
            }
        } else {
            for sink in self.sinks.iter() {
                sink.send(Changed(self.id.clone(), a.clone()));
            }
        }
    }

    pub fn lift<B>(&mut self, f: Fn(A) -> B) -> LiftedSignal<A, B> {
        let rx = self.subscribe();

        LiftedSignal {f: f, memo: None, source: rx, sinks: Vec::new()}
    }

    fn subscribe(&mut self) -> Receiver<Memoized<B>> {
        let (tx, rx) = channel();
        self.sinks.push(tx);
        return rx;
    }
}
