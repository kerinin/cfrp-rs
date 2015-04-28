use std::default::*;

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
