## Iterator Based Testing

RHDL heavily favors the use of iterators for testing designs.  Many (but not all) circuits can be tested as black boxes in the following fashion:

```badascii
              +---------+
 Inputs +---->| Circuit |+----> Outputs
              +---------+
```

This is a form of "open loop testing".  We present the circuit with a series of input items, and for each input item we see that the output matches our expectations.  It becomes more complicated when we need to close the loop between the outputs and the inputs.  But for many cases, we can use a simple open loop design.

Iterators are perfect for these use cases.  Iterators allow us to take any sequence of input values, and simulate the circuit behavior for that input value.  We then get a series of output values in the form of a new iterator, which can be used to test the output of the function.

In the case of our simple Xor gate, we can create an iterator that cycles through the list of possible inputs, and then use the `run` method of every `Circuit` to map that iterator into an iterator of output values.  We can do something simple like just print them to the console:

```rust
{{#rustdoc_include ../code/src/xor.rs:xor-step-12}}
```

Here we show a chain of iterators.  Let's break it down:

- `inputs.into_iter()` is an iterator that yields `(bool, bool)`
- `.cycle()` converts it into a repeating loop - this is handy to get the extra element at the end of the test cycle
- `.take(5)` takes exactly 5 elements from the iterator
- `.map(signal)` applies the `signal` function to each, so that we get `Signal<(bool, bool), Red>` (where the domain `Red` is inferred from the type signature of the XorGate)
- `.uniform(100)` this advances time by 100 units with each sample.  The simulation engine operates on samples with a known time, so you need to feed it a series of `TimedSample` structs with monotonically increasing timestamps.  The `Uniform` iterator extension trait provides this functionality.

Running the test produces the following output:

<!-- cmdrun to-html "cd ../code &&cargo test --package code --lib -- xor::step_8::test_iterators --exact --nocapture" -->

RHDL then provides every `Circuit` with a method that consumes an iterator of `TimedSample<CircuitIO::I>` via the `RunExt` trait.  As you can see from the output of the test function, the output iterator of `.run(it)` yields items of type `TracedSample<CircuitIO::I, CircuitIO::O>`.  Let's upgrade our iterator test to check that the output meets our definition of an XorGate.  

```rust
{{#rustdoc_include ../code/src/xor.rs:xor-step-13}}
```

To break down the `for_each` argument, we have a `TracedSample<I,O>`.  To get the input, we compute

- `s.input` extracts the Input from the tuple
- `s.input.val()` extracts the `(bool, bool)` value from the `Signal<(bool, bool), Red>`
- `s.output.val()` extracts the `bool` value from the output `Signal<bool, Red>`

<!-- cmdrun to-html "cd ../code && cargo test --package code --lib -- xor::step_8::test_iterators_expected --exact --nocapture" -->

Tests like these are extremely helpful to verify that your circuitry is working as expected, and much easier than writing Verilog testbenches or studying traces.  There are occaisons when you may need to do that, so let's continue.
