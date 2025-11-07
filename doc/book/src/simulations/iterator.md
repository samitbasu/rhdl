# Iterator Based Simulation

Iterators make for excellent open loop simulation and test code.  In the simple Xor gate [example](../xor_gate/iterator_based_testing.md), we have the following test function:

```rust
#[test]
fn test_iterators() -> miette::Result<()> {
    let inputs = [(false, false), (false, true), (true, false), (true, true)];
    //                       Separate samples by 100 units - 👇
    let it = inputs.into_iter().cycle().take(5).map(signal).uniform(100);
    let uut = xor::XorGate;
    for y in uut.run(it) {
        eprintln!("{}", y);
    }
    Ok(())
}
```

The details of the Xor gate (the unit under test) are irrelevant here.  What is of note is that we start with an exhaustive list of possible inputs, and then through a series of iterator maps and transforms, we create the necessary input to drive the simulation.  In this case, we simply print the output of the simulation.  The iterator chain is as follows:

- `inputs.into_iter()` is an iterator that yields `(bool, bool)`
- `.cycle()` converts it into a repeating loop
- `.take(5)` (Brubeck!) takes exactly 5 elements from the iterator
- `.map(signal)` converts the data elements from `Digital` to `Timed` by mapping them into the `signal` function.  In this case, the domain `Red` is inferred from the type of the gate.
- `.uniform(100)` this is the only RHDL specific iterator extension used in this example, so we will look at it more carefully.

First, let us take a look at the extension trait that gives us the `run` method on our `Circuit`.  