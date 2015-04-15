/// Notes
///
/// Problems to solve:
/// 1. Ownership graph (who owns what, so everything stays in memory)
/// 2. Spawn graph (how does the spawn signal get propagated to all nodes)
/// 3. Communication graph (how do we set up the channels for data processing)
/// 4. Different coordinator's graphs must not overlap
///
/// Ownership:
/// Children own their parents if passed by value.  Parents are owned as Signal<A>
/// trait objects. When passed by reference, they're already owned, so no need.
/// 'Reactive' functions return signals by value (potentially as Signal<A> trait 
/// objects)
///
/// Spawn:
/// Parents keep immutable references to their children as Spawn trait objects.  
/// The coordinator exposes the public 'spawn' method which propagates down the 
/// tree.  As a result, parents must be passed mutably when children are created
/// so that their Vec<Spawn> can be updated
/// NOTE: Conflict!  Parents can keep immutable child refs AND be pass children
/// mutable to THEIR children
///
/// Communication:
/// Everyone has a single Rx and a Vec<Tx>.  Again, parents must be mutable so
/// these vectors can be updated.
///
/// Overlap:
/// All nodes keep an immutable reference to the coordinator (note this means 
/// that the coordinator must be immutable).
///
/// Question: Can we ensure that the mutability requirements aren't a problem?
///

mod signal;

use std::thread;
use std::sync::mpsc::*;

use signal::*;

struct Coordinator;

impl Coordinator {
    // fn channel<A>(&self) -> Channel<A>;
    // fn receiver<A>(&self) -> Receiver<A>;
}

fn lift<F, A, B>(f: F, signal: &Signal<A>) -> Signal<B> where 
    F: 'static + Send + Clone + Fn(&A) -> B,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
{
    let (in_tx, in_rx) = channel();
    match signal.send_to(in_tx) {
        Err(_) => { panic!("WTF?") }
        _ => {}
    }

    let (signal_tx, signal_rx) = channel();
    thread::spawn(move || {
        loop {
            match in_rx.recv() {
                Ok(ref a) => {
                    let b = f(a);
                    match signal_tx.send(b) {
                        Err(_) => { break }
                        _ => {}
                    }
                },
                _ => { break; }
            }
        }
    });

    Signal::new(signal_rx)
}

fn lift2<F, A, B, C>(f: F, left: &Signal<A>, right: &Signal<B>) -> Signal<C> where 
    F: 'static + Send + Clone + Fn(&A, &B) -> C,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
    C: 'static + Send + Clone,
{
    let (left_tx, left_rx) = channel();
    match left.send_to(left_tx) {
        Err(_) => { panic!("WTF?") }
        _ => {}
    }

    let (right_tx, right_rx) = channel();
    match right.send_to(right_tx) {
        Err(_) => { panic!("WTF?") }
        _ => {}
    }

    let (signal_tx, signal_rx) = channel();
    thread::spawn(move || {
        loop {
            match (left_rx.recv(), right_rx.recv()) {
                (Ok(ref a), Ok(ref b)) => {
                    let c = f(a, b);
                    match signal_tx.send(c) {
                        Err(_) => { break }
                        _ => {}
                    }
                },
                _ => { break; }
            }
        }
    });

    Signal::new(signal_rx)
}

fn foldp<F, A, B>(f: F, initial: B, signal: &Signal<A>) -> Signal<B> where 
    F: 'static + Send + Clone + Fn(&B, &A) -> B,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
{
    let (in_tx, in_rx) = channel();
    match signal.send_to(in_tx) {
        Err(_) => { panic!("WTF?") }
        _ => {}
    }

    let (signal_tx, signal_rx) = channel();
    thread::spawn(move || {
        let mut state = initial;
        loop {
            match in_rx.recv() {
                Ok(ref a) => {
                    state = f(&state, a);
                    match signal_tx.send(state.clone()) {
                        Err(_) => { break }
                        _ => {}
                    }
                },
                _ => { break; }
            }
        }
    });

    Signal::new(signal_rx)
}
