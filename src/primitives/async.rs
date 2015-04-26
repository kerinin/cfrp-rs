use std::sync::mpsc::*;

use super::super::{Event, Signal, Push};
use super::fork::Run;

pub struct Async<A> {
    parent: Box<Signal<A>>,
    tx: Sender<A>,
}

impl<A> Async<A> {
    pub fn new(parent: Box<Signal<A>>, tx: Sender<A>) -> Async<A> {
        Async {
            parent: parent,
            tx: tx,
        }
    }
}

impl<A> Run for Async<A> where
    A: 'static + Send + Clone
{
    fn run(self: Box<Self>) {
        debug!("Async::run");

        let inner = *self;
        let Async { parent, tx } = inner;

        parent.push_to(Some(Box::new(AsyncPusher {tx: tx})));
    }
}

pub struct AsyncPusher<A> {
    tx: Sender<A>,
}

impl<A> Push<A> for AsyncPusher<A> where
    A: 'static + Clone + Send,
{
    fn push(&mut self, event: Event<A>) {
        debug!("Async handling Event");

        match event {
            Event::Changed(a) => {
                match self.tx.send(a) {
                    // We can't really terminate a child process, so just ignore errors...
                    _ => {},
                }
            },
            Event::Unchanged => {
                // No change, so no point in pushing...
            },
            Event::Exit => {
                // Exit should be propagated to all top-level inputs anyway, so
                // nothing to do here...
            }
        }
    }
}

#[cfg(test)] 
mod test {
    extern crate env_logger;

    use std::thread;
    use std::sync::mpsc::*;

    use super::super::super::*;

    #[test]
    fn async_delivers_events() {
        env_logger::init().unwrap();

        let (out_tx, out_rx) = sync_channel(0);

        spawn_topology(out_tx, |t, out_tx| {

            let s = t.async(t.value(1));

            t.add(s.fold(out_tx, |tx, a| {
                info!("Received {}, sending to output channel", a);

                match tx.send(a) {
                    Err(e) => { panic!("Error sending {}", e); },
                    _ => {},
                };
            }));
        });

        let out = match out_rx.recv() {
            Err(e) => panic!("Failed to receive: {}", e),
            Ok(v) => v,
        };
        info!("Received {} from output channel, thread about to go out of scope...", out);

        assert_eq!(out, 1);
    }

    /*
     * Async is spamming - need to think through backpressure...
    #[test]
    fn async_delivers_events_asynchronously() {
        env_logger::init().unwrap();

        let (out_tx, out_rx) = sync_channel(0);
        let (in_tx, in_rx) = sync_channel(0);

        spawn_topology((in_rx, out_tx), |t, (in_rx, out_tx)| {

            // Blocking operation
            let s = t.async(t.value(1).lift(|_| { thread::sleep_ms(100000)}));

            // Lift2-ed signal - shouldn't wait for the results of s when data 
            // comes in on in_rx
            t.add(s
                  .lift2(t.listen(0, in_rx), |_, _| {})
                  .fold(out_tx, |tx, _| {
                      match tx.send(()) {
                          Err(e) => { panic!("Error sending {}", e); },
                          _ => {},
                      };
                  })
            );
        });

        in_tx.send(0);
        let out = match out_rx.recv() {
            Err(e) => panic!("Failed to receive: {}", e),
            Ok(v) => v,
        };

        assert!(true);
    }
    */
}
