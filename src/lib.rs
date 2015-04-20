mod channel;
mod fold;
mod fork;
mod input;
mod lift;
mod reactive;
mod topology;

// NOTE: See how much of this we can keep private...
pub use channel::Channel;
pub use lift::Lift;
pub use fold::Fold;
pub use input::{Input, CoordinatedInput, NoOp};
pub use fork::{Fork, Branch};
pub use reactive::Reactive;
pub use topology::{Topology, Builder};

pub trait Signal<A>
{
    fn recv(&self) -> Option<A>;
}

pub trait Run: Send {
    fn run(self: Box<Self>);
}


#[cfg(test)] 
mod test {
    // extern crate quickcheck;
    use std::sync::mpsc::*;
    
    use super::*;

    #[test]
    fn integration() {
        let (in_tx, in_rx) = channel();
        let (out_tx, out_rx) = channel();

        Topology::build( (in_rx, out_tx), |t, (in_rx, out_tx)| {

            let channel = t.channel(in_rx);
            let lift = channel.lift(|i| -> usize { i + 1 });
            let fold = lift.foldp(out_tx, |tx, a| { tx.send(a); });
            t.add(Box::new(fold));

            // t.add(Box::new(
            //     plus_one.
            //         lift(|i: &usize| -> usize { i + 1 })
            // ));
        }).run();

        in_tx.send(0usize);

        println!("Received {}", out_rx.recv().unwrap());
    }
}
