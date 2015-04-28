use std::sync::mpsc::*;

use super::super::{Event, Signal, SignalType, Push};
use super::fork::Run;

pub struct Async<A> {
    parent: Box<Signal<A>>,
    tx: SyncSender<A>,
}

impl<A> Async<A> {
    pub fn new(parent: Box<Signal<A>>, tx: SyncSender<A>) -> Async<A> {
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

        match parent.initial() {
            SignalType::Constant(_) => return,
            SignalType::Dynamic(_) => {
                parent.push_to(Some(Box::new(AsyncPusher {tx: tx})));
            },
        }
    }
}

pub struct AsyncPusher<A> {
    tx: SyncSender<A>,
}

impl<A> Push<A> for AsyncPusher<A> where
    A: 'static + Clone + Send,
{
    fn push(&mut self, event: Event<A>) {

        match event {
            Event::Changed(a) => {
                debug!("Async handling Event Changed");
                match self.tx.send(a) {
                    // We can't really terminate a child process, so just ignore errors...
                    _ => {},
                }
            },
            Event::Unchanged => {
                debug!("Async handling Event Unchanged");
                // No change, so no point in pushing...
            },
            Event::Exit => {
                debug!("Async handling Event Exit");
                // Exit should be propagated to all top-level inputs anyway, so
                // nothing to do here...
            }
        }
    }
}

/*
#[cfg(test)] 
mod test {
    extern crate env_logger;

    use std::sync::mpsc::*;

    use super::super::topology::{Topology, Builder};

    #[test]
    fn async_delivers_events() {
        env_logger::init().unwrap();

        let (in_tx, in_rx) = sync_channel(0);
        let (out_tx, out_rx) = sync_channel(0);

        let b = Builder::new();
        let input = b.listen(0usize, in_rx);
        let a = b.async(input);
        b.add(a);
        // let f = a.fold(out_tx, |tx, a| {
        //     info!("Received {}, sending to output channel", a);
        //
        //     match tx.send(a) {
        //         Err(e) => { panic!("Error sending {}", e); },
        //         _ => {},
        //     };
        // });
        // b.add(f);

        Topology::new(b).run();

        in_tx.send(1usize);
        let out = match out_rx.recv() {
            Err(e) => panic!("Failed to receive: {}", e),
            Ok(v) => v,
        };
        // info!("Received {} from output channel, thread about to go out of scope...", out);

        // assert_eq!(out, 1);
        assert!(true);
    }

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
}
*/
