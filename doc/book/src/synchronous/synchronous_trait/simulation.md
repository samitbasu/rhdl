# Simulation

The `sim` method for synchronous circuits is notable different from the case of `Circuit` because of the introduction of the clock and reset signals into the argument list:

```rust
//                 👇 - extra argument
fn sim(&self, clock_reset: ClockReset, input: Self::I, state: &mut Self::S) -> Self::O;
```

Roughly, this translates into the following with `#[derive(Synchronous)]`:

```rust
fn sim(&self, clock_reset: ClockReset, input: Self::I, state: &mut Self::S) -> Self::O {
    //                                                       👇 - different!
    let update_fn = <<Self as SynchronousIO>::Kernel as DigitalFn3>::func();
    //            👇 Not configurable...
    for _ in 0..MAX_ITERS {
        // Requires S: Clone   👇
        let prev_state = state.clone();
        //                                           👇 - extra argument
        let (outputs, internal_inputs) = update_fn(clock_reset, input, state.0);
        state.0.child1 = self.child1.sim(clock_reset, internal_inputs.child1, &mut state.1);
        state.0.child2 = self.child2.sim(clock_reset, internal_inputs.child2, &mut state.2);
        // etc.
        //      👇 Requires S: PartialEq
        if state == prev_state {
            return outputs;
        }
    }
    // Panic!  Circuit is oscillating!
}
```

The operation of this function is essentially identical to the one in `Circuit` described [here](../../circuits/circuit_trait/simulation.md), with the main difference that the clock and reset signal is passed to the kernel (which now has 3 arguments instead of 2), as well as to all of the child elements.

There is nothing inherently "synchronous" in this simulation loop.  I make no assumptions about how the state of the circuit changes in response to the clock and reset lines.  Instead the synchronous part is expressed in two ways:

- The clock and reset signals are fanned out to all internal components and are treated like global inputs that are the same at every point in the circuit hierarchy.
- The inputs and outputs are assumed to change synchronously to the clock.  As long as all elements are `Synchronous`, this invariant is enforced.  You can violate it by writing your own black box Verilog modules, doing type crimes, or simply lying to RHDL and claiming an input is synchronous when it isn't.  Bad stuff will happen.



