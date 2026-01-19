# Simulation

It is quite simple to "roll" your own simulation once you have a circuit.  Recall from the definition of the `Circuit` trait:

```rust
{{#rustdoc_include ../code/src/circuits/traits.rs:circuit_trait}}
```

Thus, for any struct `T` that `impl Circuit`, we can write a simulation loop that looks like:

```rust
let uut = T::new(); // 👈 or however you get an instance...
let mut state = uut.init();
loop {
    let i = <next_input of type T::I>;
    let o = uut.sim(i, &mut state);
    // Report value of `o`
    // Decide when to stop
}
```

In many cases, the simulation process fits nicely with Rust iterator patterns.  And there are extension traits that make using iterators easy for simulation.  We will cover simulation and iterators in a later section.

