# Concurrent Functional Reactive Programming (for Rust)

Highly influenced by [Elm](http://elm-lang.org/) - provides a framework for describing & executing 
concurrent data flow processes.

If you're not familiar with Elm or the design behind Evan Czaplicki's 
Concurrent FRP, you should read [Elm: Concurrent FRP for Functional
GUIs](http://elm-lang.org/papers/concurrent-frp.pdf) or watch [his talk at
StrangeLoop 2014](https://www.youtube.com/watch?v=Agu6jipKfYw).

This codebase is larger and more complex than some similar libraries
([frp-rust](https://github.com/tiffany352/frp-rust),
[carboxy](https://github.com/aepsil0n/carboxyl),
[rust-frp](https://github.com/glaebhoerl/rust-frp) etc) becasue it handles
concurrency (and because I'm a Rust newb).  Simplification suggestions welcome!


## Example

```rust
// create some channels for communicating with the topology
let (in_tx, in_rx) = channel();
let (out_tx, out_rx) = channel();

// Topologies are statically defined, run-once structures.  Due to how
// concurrency is handled, changes to the graph structure can cause
// inconsistencies in the data processing
// 
// You can pass some state in (here we're passing `(in_rx, out_rx)`) if you need
// to.
Topology::build( (in_rx, out_tx), |t, (in_rx, out_tx)| {

    // Create a listener on `in_rx`.  Messages received on the channel will be
    // sent to any nodes subscribed to `input`
    let input = t.add(t.listen(in_rx));

    // Basic map operation.  Since this is a pure function, it will only be
    // evaluated when the value of `input` changes
    let plus_one = t.add(input.lift(|i| -> usize { i + 1 });

    // The return value of `add` implements `Clone`, and can be used to
    // 'fan-out' data
    let plus_two = plus_one.clone().lift(|i| -> usize { i + 2 });

    // We can combine signals too.  Since it's possible to receive input on one
    // side but not the other, `lift2` always passes `Option<T>` to its
    // function.  Like `lift`, this function is only called when needed
    let combined = plus_one.lift2(plus_two, |i, j| -> usize {
        println!("lifting");
        match (i, j) {
            (Some(a), Some(b)) => a + b,
            _ => 0,
        } 
    });

    // `fold` allows us to track state across events.  Since this is assumed to
    // be impure, it is called any time a signal is received upstream of
    // `combined`.
    let accumulated = combined.fold(0, |sum, i| { sum + i });

    // Make sure to add transformations to the topology - if it's not added it
    // won't be run...
    t.add(accumulated);


// Finish the topology and run it
}).run()
```

## TODO

* implement async, liftn const, etc

* return termination handle from topology `run`
* (more) tests
* publish, travis, post
* Try to eliminate constraints
