# Glitch Check

The `glitch_check` probe can be used to verify that the outputs of a synchronous circuit do not change state except on clock edges (to within a specified time tolerance).  This is useful for ensuring that the circuit behaves correctly and does not produce unintended glitches in its output.

The relevant trait method is provided by the `ProbeExt` extension trait.  The signature is:

```rust
{{#rustdoc_include ../../code/src/probes.rs:glitch-trait}}
```

To use it, you need to provide a closure that extracts the clock signal and the signal to check from the `TracedSample`.  If the computed signal changes state at a time that is not within 1 time unit of a clock edge, then the probe will panic, indicating a glitch has been detected.

To see an example, consider the sequence of `TracedSample<(ClockReset, b8), b8>` values shown in this table:

{{#include ../../code/glitch_check_input.txt}}

Note that at time `t = 55`, an errant signal change occurs that is not on a clock edge.  In this example, I had to insert the symbol into the sequence manually, as normally RHDL circuits do not glitch.  To check for glitches, we can use the following Rust code:

```rust
{{#rustdoc_include ../../code/src/probes.rs:glitch-check-demo}}
```

When this code is run, it will panic with a message indicating that a glitch was detected at time `55`:

<!-- cmdrun to-html "cd ../../code && cargo test --lib -- probes::glitch_check_test::test_glitch_check_probe --exact --nocapture 2>&1" -->

