# Extension Traits

There are a number of [extension traits](http://xion.io/post/code/rust-extension-traits.html) that make working with iterators easier.  In particular, there are two idiomatic patterns for working with iterators that make writing test cases simpler and more compact.  

When working with asynchronous circuits, it is generally useful to build iterator pipelines that look like this:

```badascii
                                TimedSample<(I,O)>     
                                     +             
+--------+   +---------+   +-------+ v  +---------+
| Input  |   | Uniform |   |  UUT  |    | Output  |
| Values +-->|         +-->|I     O+--->| Checker |
|        |   |         |   |       |    |         |
+--------+ ^ +---------+ ^ +-------+    +---------+
           +             +                         
   Item:   I          TimedSample<I>               
```

Here, a series of input values of type `I` are passed through a `Uniform` block that generates a series of equally spaced time samples.  These then pass through the unit under test (using the `.run()` method), and are then checked by some output logic.  With extension traits, this looks something like

```rust
let outputs = uut.run(inputs.uniform(N)); // Yields Item=TimedSample<(I,O)>
```

When working with synchronous circuits in [open loop](../open_loop.md) testing, you will generally build test pipelines like this:

```badascii
                                     TimedSample<(ClockReset,I,O)>
                                                   +                
+--------+   +-------+   +-------+       +-------+ v  +---------+   
| Input  |   | With  |   | Clock |       |  UUT  |    | Output  |   
| Values +-->| Reset +-->| Pos   +---+-->|I     O+--->| Checker |   
|        |   |       |   | Edge  |   +-->|cr     |    |         |   
+--------+ ^ +-------+ ^ +-------+ ^     +-------+    +---------+   
           +           +           +                                
   Item:   I    ResetOrData<I>  TimedSample<(ClockReset,I)>         
```

Here, a series of input values of type `I` are passed into a `WithReset` block that generates either reset signals or input data, and is generally used to prepend a number of reset pulses onto the beginning of the test data.  Next the `ResetOrData` items are passed into the `ClockPosEdge` block, which generates timed samples containing the clock information.  After testing, the input is augmented with the simulated output of the circuit using the `.run` method, and fed to the output checker.  Again, in code, this looks something like

```rust
/// 👇 Iterator with Item=TimedSample<(ClockReset, I, O)>
let outputs = uut.run(inputs.with_reset(1).clock_pos_edge(100));
```

These compact forms are possible due to a set of extension traits that are helpful for working with open loop testing.  In summary, these are:

- `.run`, which converts any `impl Circuit` or `imply Synchronous` into an iterator-driven simulator.
- `.unform(n)`, which converts a sequence of values into a sequence of `TimedSample` with equi-spaced time values.
- `.with_reset(n)`, which prepends a set of `n` reset pulses on to a data sequence.
- `.clock_pos_edge(n)`, which converts a sequence into a clock, reset and data sequence such that changes occur only after the positive edge of the clock.
- `.merge`, which merges two different timed streams into a single time stream.

We will cover each of these in detail in the following.
