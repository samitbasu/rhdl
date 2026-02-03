# Sample At Neg Edge

When working with synchronous circuits, you often want the output sequence to be compared by some test logic against an expected sequence.  In almost all instances, a synchronous circuit will generate outputs on clock edges that follow the input that is sampled on the previous clock edge.  By means of illustration:

```badascii
              +------+      +------+       
clock         |      |      |      |       
        +-----+      +------+      +------+
                                           
input   ++I0+-+----+I1+-----+-----+I2+----+
        +-----+-------------+-------------+
                                           
          ^              ^                 
          +---------+    +---------+       
                    v              v       
                                           
output  +-----+----+O0+-----+-----+O1+----+
        +-----+-------------+-------------+
```

Here the input sequence `I0, I1, I2...` is sampled on the rising edge of the clock, and the output sequence `O0, O1, O2,...` is generated on the same rising edge of the clock at some short time after the rising edge.  Note, however, that the output `O0`, is available only _after_ the rising edge of the clock.  Which makes it tricky to sample the output sequence to extract the `O0`, `O1` values.

You generally have a couple of choices:

- Sample synchronously with the rising edge of the clock, and then adjust for the latency of the circuit under test.  In this example, if we sampled on the rising edge, the first output would be undefined, so we would need to skip it.
- Sample on the _falling_ edge of the clock, which will give you the output values `O0, O1, ...` at the correct times, assuming the circuit has a single clock cycle latency.

The `sample_at_neg_edge` probe provides a convenient way to sample the output of a synchronous circuit at the falling edge of the clock.  The relevant trait method is provided by the `ProbeExt` extension trait.  The signature is:

```rust
{{#rustdoc_include ../../code/src/probes.rs:sample-at-neg-edge-trait}}
```

To use it, you need to provide a closure that extracts the clock signal from the `TracedSample`.  The probe will yield only those samples that occur at the falling edge of the clock.

```rust
{{#rustdoc_include ../../code/src/probes.rs:sample-at-neg-edge-demo}}
```

For reference, is the trace of this simulation which is simply a single 8-bit wide digital flip-flop:

![Sample At Neg Edge Trace](../../code/dff_sample_at_neg_edge.svg)


The iterator produces only a set of values at the falling edge of the clock, and discards all others.  

{{#include ../../code/dff_sample_at_neg_edge.txt}}
