# Output Passthrough

In a manner completely analogous to input passthrough drivers, output passthrough drivers can be used to connect a circuit output to a top-level module output port with no circuitry in between the two.  This allows you to expose circuit outputs as top-level module outputs.

```rust
{{#rustdoc_include ../code/src/fixturing.rs:fixture-with-io}}
```

We now have an instance of the output type of the circuit in binding `output`.  We use the `path!` macro to indicate which fields of the output struct we want to connect to top-level module output ports.  The updated generated Verilog is as follows:

```verilog
{{#include ../code/and_fixture_step_3.v}}
```

Note that the top level module now has an output port `out` that is connected directly to the output of the AND gate.
