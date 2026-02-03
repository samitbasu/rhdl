# Iterator Based Simulation

Iterators make for excellent open loop simulation and test code.  In the simple Xor gate [example](../xor_gate/iterator_based_testing.md), we have the following test function:

```rust
{{#rustdoc_include ../code/src/simulations.rs:test_iterators}}
```

The result of running this code is all of the test cases written to a file:

```text
{{#include ../code/xor_trace.txt}}
```

The details of the Xor gate (the unit under test) are irrelevant here.  What is of note is that we start with an exhaustive list of possible inputs, and then through a series of iterator maps and transforms, we create the necessary input to drive the simulation.  In this case, we simply print the output of the simulation.  The iterator chain is as follows:

- `inputs.into_iter()` is an iterator that yields `(bool, bool)`
- `.map(signal)` converts the data elements from `Digital` to `Timed` by mapping them into the `signal` function.  In this case, the domain `Red` is inferred from the type of the gate.
- `.uniform(100)` this is the only RHDL specific iterator extension used in this example, so we will look at it more carefully.



