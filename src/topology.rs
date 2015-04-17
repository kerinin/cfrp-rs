use std::thread::*;

use super::coordinator::{RunChannel, Coordinator};
use super::signal::Run;

pub struct Topology {
    pub coordinator: Coordinator,
    pub signals: Vec<Box<Run>>,
}

impl Topology {
    pub fn run(self) {
        /*
        let moved_coordinator = self.coordinator;
        spawn(move || {
            moved_coordinator.run()
        });

        for signal in self.signals.into_iter() {
            spawn(move || {
                signal.run()
            });
        }
        */
    }
}
