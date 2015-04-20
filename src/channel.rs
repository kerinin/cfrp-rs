use std::sync::mpsc::*;

use super::{Channel, Signal};

impl<A> Signal<A> for Channel<A> where
    A: 'static + Send,
{
    fn recv(&self) -> Option<A> {
        match self.source_rx.recv() {
            Err(_) => None,
            Ok(a) => a,
        }
    }
}
