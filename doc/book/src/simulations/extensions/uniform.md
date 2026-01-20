# Uniform

Generating a timed sequence from a regular sequence can be done in a couple of different ways.  The simplest is to use `.map` with a closure.  Something like:

```rust
{{#rustdoc_include ../../code/src/simulations.rs:uniform-with-map}}
```

The resulting iterator will yield the following sequence of `TimedSample<Q>` items:

{{#include ../../code/uniform_map.txt}}

In this case the number `50` is arbitrary.  It represents some time interval between the changes to the input signal.  I generally choose something on the order of ~100 because it makes viewing the signals in a trace viewer easier.  But the number is (for RHDL) completely arbitrary.  If you want to think in nanoseconds or picoseconds, you can use a different number here.  

This pattern is frequent enough that there is an extension trait for creating a set of uniformly spaced `TimedSample<Q>` from any iterator that produces items of type `Q`.  The extension trait definition is

```rust
{{#rustdoc_include ../../code/src/simulations.rs:uniform_ext}}
```

and creates an iterator that yields items of type `TimedSample<Q>` from an iterator that produces items of type `Q` where `Q: Digital`.  So our previous snippet can be shortened to:

```rust
{{#rustdoc_include ../../code/src/simulations.rs:uniform-without-map}}
```

{{#include ../../code/uniform_map_ext.txt}}

Once the iterator is consumed by the `.run` method, the output contains items of type `TracedSample<I,O>`.  The reason for this is that by passing the input on to the output, it is easier to write assertions that use some other means to independently verify the operation of the circuit.  For example, in our trivial Xor [gate](../../xor_gate/iterator_based_testing.md), we use a test like this:

```rust
{{#rustdoc_include ../../code/src/simulations.rs:xor-uniform-iter}}
```

Here, the output of `uut.run(it)` is a sequence of `TracedSample<I,O>` items.  These are then unpacked as:

- `s` is of type `TimedSample<(I,O)>`
- `s.value` is the value of the `TimedSample` (we do not use the time in this test), and has type `(I,O)`
- `s.value.0` is the input fed to the circuit at the corresponding timestamp, and has type `I`, which for this circuit, is `Signal<(bool, bool), Red>`.
- `s.value.0.val()` is the value carried by the signal (remember that `Signal` is just a wrapper type), so this is of type `(bool, bool)`, which is the input
- `s.value.1.val()` is by the same argument, the output of the circuit, and is of type `bool`, the output of the circuit.
- We can then compute the Xor of the two input bits using the Rust `^` operator to yield `expected`
- An assertion then compares the circuit output to the Rust output

This example demonstrates the general flow for open loop testing using iterators and the extension traits.  In general, open loop tests will look like this:

- We create an iterator that yields inputs of interest to present to our circuit
- We use `.uniform` to put these inputs on a uniform time grid for feeding to our unit under test (UUT)
- The `.run()` method consumes the iterator, and produces a sequence of timed samples that contain the input and the output
- Some independent means is used to compute the expected output of the circuit to the input provided.  This independent means can use _any_ valid Rust code to do so.
- The output of the circuit is checked against the independently computed output and any discrepancies cause a failure of the test.

You can even do fuzz-type testing with this setup, where the input is randomly generated, and the output is checked for correctness, terminating only after some amount of CPU time has elapsed.  I can imagine that for quite complicated circuits with large input spaces, this might be a useful technique.  


```admonish note
There is no assumption of stateless-ness in this test setup.  The circuit will almost certainly have internal state, and that state may depend on the precise sequence/order of inputs presented.  When testing stateful designs, you have to consider how the sequence of inputs will affect the state of the circuit.  I have found it easier to test the transition function directly,  using the [exhaustive testing](./exhaustive_testing.md) method, since you can force the input and current state of the circuit to all possibilities.
```

