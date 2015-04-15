use std::thread;
use std::cell::*;
use std::rc::*;
use std::sync::mpsc::*;

trait Spawn {
    fn spawn(self);
}

impl<T> Spawn for Rc<RefCell<T>> where T: Spawn {
    fn spawn(self) {
        (*self).into_inner().spawn()
    }
}

pub struct Coordinator {
    nodes: Vec<Box<Spawn>>,
}

impl Coordinator {
    pub fn coordinate(&mut self, node: Box<Spawn>) {
        self.nodes.push(node)
    }

    // fn channel() -> Channel
    // fn run()
    // rn terminate()
}
