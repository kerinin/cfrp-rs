use std::sync::mpsc::channel;

pub struct Reactor {
}

impl Reactor {
    pub fn channel<T>() -> (Channel<T>, Signal<T>) {
        let (tx, rx) = channel();
        let mut channel = Channel {tx: tx};

        let (signal, rx) = Signal::spawn()
        let signal = Signal {};
        channel.signals.push(signal);

        thread::spawn(move || {
            channel.run(rx)
        });

        return (channel, signal);
    }

    pub fn async<T>(Signal<T>) -> Signal<T>) {
    }

    pub fn lift<A, B>(f: Fn(A) -> B, a: Signal<A>) -> Signal<B> {
        let rx = a.listen();
        let (signal, tx) = Signal::spawn();

        thread::spawn(move || {
            for msg in rx.iter() {
                tx.send(f(msg));
            }
        });

        return signal;
    }

    pub fn foldp<A, B>(Fn(B, A) -> B, B, Signal<A>) -> Signal<B> {
    }
}

impl Default for Reactor {
    fn default() -> Reactor {
        Reactor {}
    }
}
