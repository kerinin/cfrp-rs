// mod signal;
mod signal2;
// mod coordinator;
mod coordinator2;

use std::thread;
use std::sync::mpsc::*;

use signal2::*;

fn lift<'a, F, A, B>(f: F, signal: &Signal<'a, A>) -> Signal<'a, B> where 
    F: 'static + Send + Clone + Fn(&A) -> B,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone + Eq,
{
    // NOTE: Check for coordinator match
    let (in_tx, in_rx) = channel();
    signal.publish_to(in_tx);

    let (signal_tx, signal_rx) = channel();
    thread::spawn(move || {
        let mut last_b = None;
        let mut send_if_changed = |a: &A| -> bool {
            let b = Some(f(a));

            let value = if last_b == b {
                None
            } else {
                last_b = b.clone();
                b
            };

            match signal_tx.send(value) {
                Err(_) => { true }
                _ => { false }
            }
        };

        loop {
            match in_rx.recv() {
                // Signal value changed
                Ok(Some(ref a)) => {
                    if send_if_changed(a) { break }
                }
                
                // No change from previous - send it along!
                Ok(None) => {
                    match signal_tx.send(None) {
                        Err(_) => { break }
                        _ => {}
                    }
                }

                // Receiver closed, time to pack up & go home
                _ => { break; }
            }
        }
    });

    Signal::new(&signal.coordinator, signal_rx)
}

fn lift2<'a, F, A, B, C>(f: F, left: &Signal<'a, A>, right: &Signal<'a, B>) -> Signal<'a, C> where 
    F: 'static + Send + Clone + Fn(&A, &B) -> C,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone,
    C: 'static + Send + Clone + Eq,
{
    let (left_tx, left_rx) = channel();
    left.publish_to(left_tx);

    let (right_tx, right_rx) = channel();
    right.publish_to(right_tx);

    let (signal_tx, signal_rx) = channel();
    thread::spawn(move || {
        let mut last_a = None;
        let mut last_b = None;
        let mut last_c = None;
        let mut send_if_changed = |a: &A, b: &B| -> bool {
            let c = Some(f(a, b));

            let value = if last_c == c {
                None
            } else {
                last_c = c.clone();
                c
            };

            match signal_tx.send(value) {
                Err(_) => { true }
                _ => { false }
            }
        };

        loop {
            match (left_rx.recv(), right_rx.recv()) {
                (Ok(Some(ref a)), Ok(Some(ref b))) => {
                    if send_if_changed(a, b) { break };
                }

                (Ok(None), Ok(Some(ref b))) => {
                    let a = match last_a.clone() {
                        Some(a) => { a }
                        None => { panic!("Channel reports no change, but nothing was cached") }
                    };
                    if send_if_changed(a, b) { break };
                }

                (Ok(Some(ref a)), Ok(None)) => {
                    let b = match last_b.clone() {
                        Some(b) => { b }
                        None => { panic!("Channel reports no change, but nothing was cached") }
                    };
                    if send_if_changed(a, b) { break };
                }

                (Ok(None), Ok(None)) => {
                    match signal_tx.send(None) {
                        Err(_) => { break }
                        _ => {}
                    }
                }

                _ => { break; }
            }
        }
    });

    Signal::new(&left.coordinator, signal_rx)
}

fn foldp<'a, F, A, B>(f: F, initial: B, signal: &Signal<'a, A>) -> Signal<'a, B> where 
    F: 'static + Send + Clone + Fn(&B, &A) -> B,
    A: 'static + Send + Clone,
    B: 'static + Send + Clone + Eq,
{
    let (in_tx, in_rx) = channel();
    signal.publish_to(in_tx);

    let (signal_tx, signal_rx) = channel();
    thread::spawn(move || {
        let mut state = initial;
        let mut last_b = None;

        let mut send_if_changed = |a: &A| -> bool {
            let b = Some(f(&state, a));

            let value = if last_b == b {
                None
            } else {
                last_b = b.clone();
                b
            };

            match signal_tx.send(value) {
                Err(_) => { true }
                _ => { false }
            }
        };

        loop {
            match in_rx.recv() {
                Ok(Some(ref a)) => {
                    if send_if_changed(a) { break }
                }

                Ok(None) => {
                    match signal_tx.send(None) {
                        Err(_) => { break }
                        _ => {}
                    }
                }

                _ => { break; }
            }
        }
    });

    Signal::new(&signal.coordinator, signal_rx)
}

fn async<'a, A>(signal: &Signal<'a, A>) -> Signal<'a, A> where
    A: 'static + Send + Clone,
{
    let (in_tx, in_rx) = channel();
    signal.publish_to(in_tx);

    let (channel_tx, signal) = signal.coordinator.channel();
    thread::spawn(move || {
        loop {
            match in_rx.recv() {
                Ok(Some(ref a)) => {
                    channel_tx.send(a.clone());
                }

                Ok(None) => {}

                _ => { break }
            }
        }
    });

    signal
}
