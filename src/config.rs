use std::default::*;

/// Topology Configuration
/// 
/// `buffer_size` determines the size of the SyncSender channels to use for
/// transporting events between threads.  Smaller values _may_ result in less 
/// memory consumption, larger values _may_ result in higher throughput.
///
#[derive(Clone, Debug)]
pub struct Config {
    pub buffer_size: usize,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            buffer_size: 0,
        }
    }
}
