use std::thread::*;

use super::*;
use super::coordinator::Coordinator;

pub struct Topology {
    pub coordinator: Coordinator,
    pub nodes: Vec<Box<Compile>>,
}

impl Topology {
    pub fn run(self) {
        self.coordinator.run();

        for node in self.nodes.into_iter() {
            // let moved_node = node;
            // let compiled: Box<Run> = moved_node.compile();
            // let compiled_node = node.compile();
            // spawn(move || {
            //     // compiled_node.run()
            // });
        }
    }
}
