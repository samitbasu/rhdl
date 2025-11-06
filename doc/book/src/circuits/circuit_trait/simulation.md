# Simulation

The `sim` method is responsible for calculating the output of the circuit given its current state and an input.  The circuit is allowed to mutate the state as a result of the input.  

```rust
fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O;
```

Roughly, this function translates as the following with `#[derive(Circuit)]`:

```rust
fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O {
    let update_fn = <<Self as CircuitIO>::Kernel as DigitalFn2>::func();
    //            👇 Not configurable...
    for _ in 0..MAX_ITERS {
        // Requires S: Clone   👇
        let prev_state = state.clone();
        let (outputs, internal_inputs) = update_fn(input, state.0);
        state.0.child1 = self.child1.sim(internal_inputs.child1, &mut state.1);
        state.0.child2 = self.child2.sim(internal_inputs.child2, &mut state.2);
        // etc.
        //      👇 Requires S: PartialEq
        if state == prev_state {
            return outputs;
        }
    }
    // Panic!  Circuit is oscillating!
}
```

The simulation function is a little complex, so let's break it down.  Remember, in most cases you don't have to write this yourself.  But it's good to understand how the simulation engine in RHDL works, so here goes.

1. We start with the previous state and some new input to the circuit.  We want to compute the new output and update the state in the process.
2. Recall that our circuit is structured in a feedback topology, in which outputs from the kernel are fed back into the child subcircuits.  This feedback topology is replicated at every level in the design, wherever circuits are composed.
3. The feedback means that you cannot directly compute the output from the input and the current state.  The output includes the feedback inputs to the internal components, so it takes iterations for those changes to "go around" the loops and settle to a final value.
4. If you have logic loops in your design, it's possible that these iterations will never settle, and the simulation will just oscillate.  That will cause a panic.
5. When the state stops changing (and remember the state includes the `Q` feedback to the internal circuits), we have a new state, and can accept the proposed output.

There is a maximum number of iterations that bounds the number of times we can go around the loop before deciding that things are not going to settle.  It is currently set to 10, but that number is arbitrary.  I'm sure you could construct circuits that are non-oscillatory but do not converge in 10 iterations (or any number `N` of iterations).  But these are _probably_ pathological.  And in most cases, long chains of combinatorial logic in feedback chains means you have some issues with your design anyway.

```admonish warning
If your simulation panics because of convergence issues (taking more than `MAX_ITERS` iterations to converge), consider the possibility that you have a logic loop, an oscillator or simply over-complicated logic.  Consider breaking your circuit into smaller segmented pieces.  Also consider introducing stateful components (like flip flops) into your design to make the paths shorter.
```
