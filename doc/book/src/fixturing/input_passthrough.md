# Input Passthrough

The simplest driver you can add to a fixture is a `pass_through_input` driver.  The `pass_through_input` driver connects a top-level input port to an input of the circuit with no circuitry in between the two.  It essentially allows you to connect a circuit input to a top level module input port.  

For our AND gate example, we can add two input passthrough drivers to connect the top-level inputs `a_in` and `b_in` to the circuit inputs:

```rust
{{#rustdoc_include ../code/src/fixturing.rs:fixture-with-inputs}}
```

This construct uses the `path!` macro.  The `path!` macro takes an expression meant to indicate where to connect the signal.  In this case, we create an instance of the input type for the `ANDGate` circuit using the `dont_care()` method, and then use `path!(input.val().0)` to indicate that we want to connect to the first field of the input struct (which corresponds to input `a_in`), and `path!(input.val().1)` to indicate that we want to connect to the second field of the input struct (which corresponds to input `b_in`).

If we compile this fixture, we get the following Verilog output:

```verilog
{{#include ../code/and_fixture_step_2.v}}
```

Note that the output of the AND gate goes nowhere.  This is because we have not used a driver to connect it.  The fixture compiles, however, as it is not required to utilize all outputs of a circuit, only all inputs.
