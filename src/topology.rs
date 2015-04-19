use super::{Coordinator, Spawn};

pub struct Topology {
    coordinator: Coordinator,
    nodes: Vec<Box<Spawn>>,
}

impl Topology {
    pub fn new(coordinator: Coordinator) -> Topology {
        Topology {
            coordinator: coordinator,
            nodes: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: Box<Spawn>) {
        self.nodes.push(node);
    }

    pub fn run(self) {
        self.coordinator.spawn();

        for node in self.nodes.into_iter() {
            node.spawn();
        }
    }
}
