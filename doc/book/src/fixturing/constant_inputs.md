# Constant Inputs

Sometimes you need to provide a constant input to a circuit in a fixture.  For example, you may need to provide an enable signal that is always high, or a configuration value that is hard coded.  For these, the `Fixture` struct provides a `constant_input` driver.  This driver allows you to feed a constant value of type `S: Digital` to a path on the input circuit.

The technique is illustrated here by tying the second input of our AND gate to a constant high value:

```rust
{{#rustdoc_include ../code/src/fixturing.rs:fixture-with-constant}}
```

The generated Verilog module only has a single input port `a_in`, and the second input of the AND gate is tied to constant high:

```verilog
{{#include ../code/and_fixture_step_5.v}}
```

