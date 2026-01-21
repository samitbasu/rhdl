# Run

Recall from [here](../../circuits/simulation.md) that the `Circuit` trait included a `sim` method:

```rust
{{#rustdoc_include ../../code/src/simulations.rs:run-circuit}}
```

and that a simulation loop would need to look something like this to drive the simulation:

```rust
{{#rustdoc_include ../../code/src/simulations.rs:test_sim}}
```

To facilitate tracing, we will also want to establish the time `t0` associtated with each input sample.  We do this with the [TimedSample](../time.md) struct, which associates a time with each input sample.  

RHDL provides an extension trait that gives us the `run` method on our `Circuit`:
Essentially, assuming:

- You have some thing `i` which implements `IntoIterator`, with an `Item = TimedSample<I>` where `I` is the circuit input type.
- You have a circuit `x` with input type `I` and output type `O`.
- Calling `x.run(i)` will yield a new iterator that yields items of type `TracedSample<I,O>`.

In practical terms, this means that if you can generate a sequence of timed input samples of type `I`, then the `x.run()` method will transform these into a sequence of traced samples of type `TracedSample<I,O>`, which is quite handy for testing.  To use this, you need to `use` the extension trait, `RunCircuitExt`, which is included in the prelude.

Here is the previous example, rewritten to use the extention trait:

```rust
{{#rustdoc_include ../../code/src/simulations.rs:test_sim_ext}}
```

Similarly for `impl Synchronous`, the `RunSynchronousExt` trait provides a `run` method that works similarly, but also preserves the clock and reset information.  Essentially, assuming:

- You have some thing `i` which implements `IntoIterator`, with an `Item = TimedSample<(ClockReset, I)>` where `I` is the circuit input type.  Clock and reset information are contained in the `ClockReset` part of the tuple.
- You have a synchronous circuit `x` with input type `I` and output type `O`.
- Calling `x.run(i)` will yield a new iterator that yields items of type `TracedSample<(ClockReset,I),O>`.

In practical terms, this means that if you can generate a sequence of timed input samples of type `(ClockReset, I)`, then the `x.run()` method will transform these into a sequence of traced samples of type `TracedSample<(ClockReset, I),O>`.

For completeness, here is an example of using the `RunSynchronousExt` trait:

```rust
{{#rustdoc_include ../../code/src/simulations.rs:counter-iter-ext}}
```

If the `post_process` function were to write out the output samples, you would get the following table of outputs:

{{#include ../../code/counter.txt}}

As we will see later, you can also collect the output of the `run` method into a container to generate a complete trace of the simulation.

```rust
{{#rustdoc_include ../../code/src/simulations.rs:counter-iter-svg}}
```

with a resuling SVG trace that looks like this:

![Counter Trace](../../code/counter.svg)