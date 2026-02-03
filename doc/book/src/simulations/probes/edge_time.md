# Edge Time

The `edge_time` iterator extension is provided by the `ProbeExt` extension trait.  The relevant signature is:

```rust
{{#rustdoc_include ../../code/src/probes.rs:edge-time-trait}}
```

The method (which is implemented on all iterators that yield `TracedSample` values) produces a new iterator that yields only the samples where the provided closure returns a value that changes.  For example, if you have a stream of `TracedSample<(ClockReset, I), O>` values coming from a RHDL simulation of your synchronous circuit, you can use `edge_time` to find only those samples where some critical value changes.  Furthermore, you can look for multiple different changing values by returning a tuple from the closure.

Here is a simple example.  Suppose that we have the output of a synchronous circuit that produces `b8` values from a stream of input `b8` values.  We want to find all times when the output changes.  The stream is documented in this table: 

{{#include ../../code/edge_time_input.txt}}

To find the times when the output changes, we can use the following Rust code:

```rust
{{#rustdoc_include ../../code/src/probes.rs:edge-time-demo}}
```

{{#include ../../code/edge_time_output.txt}}

As you can see, the iterator drops all samples except those where the output value changes.  The output shows the time, input, and output values at each of these change points.