mod primitives;
mod signal;
mod topology;

pub use topology::{Topology, Builder};
pub use primitives::fork::Branch;

use primitives::InternalSignal;

/// A data source of type `A`
///
pub struct Signal<A> {
    internal_signal: Box<InternalSignal<A>>,
}

#[cfg(test)] 
mod test {
    extern crate log;

    // extern crate quickcheck;
    use std::thread;
    use std::sync::mpsc::*;

    use super::*;

    #[test]
    fn integration() {
        let (in_tx, in_rx) = channel();
        let (out_tx, out_rx) = channel();

        Topology::build( (in_rx, out_tx), |t, (in_rx, out_tx)| {

            let input = t.add(t.listen(in_rx));

            // t.add(input.clone()
            //       .liftn((input,), |(i, j)| -> usize { println!("lifting"); 0 })
            //       .foldp(out_tx.clone(), |tx, a| { tx.send(a); })
            //      );
            t.add(input.clone()
                  .lift(|i| -> usize { i })
                  .lift2(input.lift(|i| -> usize { i }), |i, j| -> usize {
                      println!("lifting");
                      match (i, j) {
                          (Some(a), Some(b)) => a + b,
                          _ => 0,
                      } 
                  }).foldp(out_tx.clone(), |tx, a| { tx.send(a); })
                 );





            /*
            let plus_one = t.add(t.listen(in_rx)
                .lift(|i| -> usize { println!("lifting to plus_one"); i + 1 })
            );

            let plus_two = t.add(plus_one.clone()
                .lift(|i| -> usize { println!("lifting to plus_two"); i + 1 })
            );

            let plus_three = t.add(plus_one.clone()
                .lift(|i| -> usize { println!("lifting to plus_three"); i + 2 })
            );

            t.add(plus_two
                .liftn((plus_three,), |(i, j)| -> usize { 
                    println!("liftn-ing to lifted");

                    match (i, j) {
                        (Some(a), Some(b)) => { a + b },
                        _ => 0,
                    }
                }).foldp(out_tx, |tx, a| { tx.send(a); })
            );

            t.add(plus_two
                .foldp(out_tx.clone(), |tx, a| { tx.send(a); })
            );
            t.add(plus_three
                .foldp(out_tx.clone(), |tx, a| { tx.send(a); })
            );
            */

        }).run();

        thread::sleep_ms(1000);

        in_tx.send(1usize);

        let out = out_rx.recv().unwrap();
        assert_eq!(out, 2);
        // println!("Received {}", out);
        /*

        let out = out_rx.recv().unwrap();
        assert_eq!(out, 3);
        println!("Received {}", out);
        */
    }
}
