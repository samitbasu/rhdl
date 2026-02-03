# Bind Macro

Because input and output pass through drivers occur frequently, a macro named `bind!` is provided to make it easier to add multiple input and output passthrough drivers to a fixture.  The `bind!` macro takes a fixture instance and creates either an input pass through driver or an output passthrough driver depending on the direction of the arrow used in the macro invokation.  Here is the example of using the `bind!` macro instead of the `pass_through_input` and `pass_through_output` methods:

```rust
{{#rustdoc_include ../code/src/fixturing.rs:fixture-with-bind}}
```

The resulting Verilog is identical to the previous examples, as shown below:

```verilog
{{#include ../code/and_fixture_step_4.v}}
```

The `bind!` macro simply makes the fixture code easier to read and write by reducing the boilerplate.
