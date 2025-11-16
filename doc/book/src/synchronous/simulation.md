# Simulation

Just as in for `Circuit`, the `sim` method is responsible for simulating the circuit given it's current state and an input.  The circuit is allowed to mutate the state as a result of the input.  Unlike the `Circuit` case described [here](../circuits/simulation.md), we need to also provide the state of the `clock` and `reset` signals to the simulation.  Hence the signature of the relevant method 

```rust
pub trait Synchronous: 'static + Sized + SynchronousIO {
    type S: PartialEq + Clone;
    fn init(&self) -> Self::S;
    fn sim(&self, clock_reset: ClockReset, input: Self::I, state: &mut Self::S) -> Self::O;
    // snip
}
```

The `Synchronous` trait makes it trivial for you to simulate the circuit using a simple loop.  Roughly, a simulation loop would look something like:

```rust
let mut uut = T::new(); // Or whatever to get an instance of T : Synchronous
let mut state = T::init(); // Get the initial state
loop {
    let clock_reset = // get the next clock and reset value
    let input = // get the next input
    let output = uut.sim(clock_reset, input, &mut state);
    // Do something with the output
    // check for simulation complete.
}
```

You can build your own, but RHDL has an easier way to run simulations using iterators and extension traits.  We will cover those in their own section later.  

If you use the `#[derive(Synchronous)]` macro on your struct, then RHDL will autogenerate an implementation that looks something like this:

```rust
fn sim(&self, cr: ClockReset, input: Self::I, state: &mut Self::S) -> Self::O {
    let update_fn = <<Self as SynchronousIO>::Kernel as DigitalFn3>::func();
    for _ in 0..MAX_ITERS {
        // Requires S: Clone
        let prev_state = state.clone();
        let (outputs, internal_inputs) = update_fn(cr, input, state.0);
        state.0.child1 = self.child1.sim(cr, internal_inputs.child1, &mut state.1);
        state.0.child2 = self.child2.sim(cr, internal_inputs.child2, &mut state.2);
        // etc.
        if state == prev_state {
            return outputs;
        }
    }
    // Panic! Circuit is oscillating!
}
```

The only substantive difference for `Synchronous` vs `Circuit` `sim` methods is the injection of a clock reset that is passed into all subcircuits.  This global injection of the clock and reset helps automate an otherwise error-prone and repetitive task (namely feeding the clock and reset signals manually into all of the subcircuits through the hierarchy).