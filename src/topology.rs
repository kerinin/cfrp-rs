use std::thread::*;

use super::*;
use super::coordinator::Coordinator;

pub struct Topology {
    pub coordinator: Coordinator,
    pub nodes: Vec<Box<Run>>,
}

impl Topology {
    pub fn run(self) {
        self.coordinator.run();

        for node in self.nodes.into_iter() {
            spawn(move || {
                node.run();
            });
        }
    }
}
