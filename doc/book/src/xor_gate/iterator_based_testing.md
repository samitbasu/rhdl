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

```rust,write:xor/tests/test_iterator.rs
use rhdl::prelude::*;

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

Here we show a chain of iterators.  Let's break it down:

- `inputs.into_iter()` is an iterator that yields `(bool, bool)`
- `.cycle()` converts it into a repeating loop - this is handy to get the extra element at the end of the test cycle
- `.take(5)` takes exactly 5 elements from the iterator
- `.map(signal)` applies the `signal` function to each, so that we get `Signal<(bool, bool), Red>` (where the domain `Red` is inferred from the type signature of the XorGate)
- `.uniform(100)` this advances time by 100 units with each sample.  The simulation engine operates on samples with a known time, so you need to feed it a series of `TimedSample` structs with monotonically increasing timestamps.  The `Uniform` iterator extension trait provides this functionality.

```shell,rhdl:xor
cargo build -q
cargo test --test test_iterator -- --nocapture
```  

RHDL then provides every `Circuit` with a method that consumes an iterator of `TimedSample<CircuitIO::I>` via the `RunExt` trait.  As you can see from the output of the test function, the output iterator of `.run(it)` yields items of type `TimedSample<(CircuitIO::I, CircuitIO::O)>`.  Let's upgrade our iterator test to check that the output meets our definition of an XorGate.  

```rust,write:xor/tests/test_iterator_expected.rs
use rhdl::prelude::*;

#[test]
fn test_iterators() -> miette::Result<()> {
    let inputs = [(false, false), (false, true), (true, false), (true, true)];
    let it = inputs.into_iter().cycle().take(5).map(signal).uniform(100);
    let uut = xor::XorGate;
    //                   👇 TimedSample<(Signal<(bool,bool), Red>, Signal<bool, Red>)>
    uut.run(it).for_each(|s| {
        let input = s.value.0.val();
        let output = s.value.1.val();
        let expected = input.0 ^ input.1;
        assert_eq!(output, expected, "For input {input:?}, expected {expected}");
    });
    Ok(())
}
```

To break down the `for_each` argument, we have a `TimedSample<(I,O)>`.  To get the input, we compute

- `s.value` extracts the value from the `TimedSample`
- `s.value.0` extracts the Input from the tuple
- `s.value.0.val()` extracts the `(bool, bool)` value from the `Signal<(bool, bool), Red>`
- `s.value.1.val()` extracts the `bool` value from the output `Signal<bool, Red>`

```shell,rhdl:xor
cargo build -q
cargo test --test test_iterator_expected
```

Tests like these are extremely helpful to verify that your circuitry is working as expected, and much easier than writing Verilog testbenches or studying traces.  There are occaisons when you may need to do that, so let's continue.
