# Concurrent Functional Reactive Programming (for Rust)

Concurrent FRP implemented in Rust.

![Build Status](https://travis-ci.org/kerinin/cfrp-rs.svg)

[Documentation](http://kerinin.github.io/cfrp-rs/cfrp)


If you're not familiar with Elm or the design behind Evan Czaplicki's 
Concurrent FRP, you should read [Elm: Concurrent FRP for Functional
GUIs](http://elm-lang.org/papers/concurrent-frp.pdf) or watch [his talk at
StrangeLoop 2014](https://www.youtube.com/watch?v=Agu6jipKfYw).

This codebase is larger and more complex than some similar libraries
([frp-rust](https://github.com/tiffany352/frp-rust),
[carboxyl](https://github.com/aepsil0n/carboxyl),
[rust-frp](https://github.com/glaebhoerl/rust-frp) etc) becasue it handles
concurrency (and because I'm a Rust newb).  Simplification suggestions welcome!


## TODO

* configure buffer
* docs
* SignalExt helper methods (map, zip, enumerate, etc)
* Try to eliminate constraints
* Timers, counters, rngs
