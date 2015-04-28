use std::sync::*;
use std::thread;

use super::Run;
use primitives::input::{RunInput, NoOp};


/// `Topology<T>` describes a data flow and controls its execution
///
pub struct Topology {
    inputs: Vec<Box<RunInput>>,
    runners: Vec<Box<Run>>,
}

impl Topology {
    /// Create a new topology
    ///
    pub fn new(inputs: Vec<Box<RunInput>>, runners: Vec<Box<Run>>) -> Self {
        Topology { inputs: inputs, runners: runners }
    }

    /// Run the topology
    ///
    pub fn run(self) -> TopologyHandle {
        info!("----> TOPOLOGY STARTING");
        let Topology {inputs, runners} = self;

        for runner in runners.into_iter() {
            thread::spawn(move || {
                runner.run();
            });
        }

        let no_ops = Arc::new(Mutex::new(inputs.iter().map(|i| i.boxed_no_op()).collect::<Vec<Box<NoOp>>>()));
        let term_txs = inputs.iter().map(|i| i.boxed_no_op()).collect::<Vec<Box<NoOp>>>();
        for (idx, input) in inputs.into_iter().enumerate() {
            let no_ops_i = no_ops.clone();
            thread::spawn(move || {
                input.run(idx, no_ops_i);
            });
        }

        info!("----> TOPOLOGY RUNNING...");

        TopologyHandle {
            term_txs: term_txs,
        }
    }
}

/// For explicitly terminating a running topology
///
pub struct TopologyHandle {
    term_txs: Vec<Box<NoOp>>,
}

// NOTE: Drop?  seems to kill tests for some reason, maybe because not capturing
impl TopologyHandle {
    pub fn stop(&mut self) {
        for tx in self.term_txs.iter() {
            tx.send_exit();
        }
        debug!("----> TOPOLOGY DROPPED");
    }
}
