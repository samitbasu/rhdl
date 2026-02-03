# Synthesis

For now, getting a `impl Circuit` onto physical hardware requires translation into a well accepted standardized Hardware Description Language.  Currently, that takes the form of a stripped down subset of Verilog.  Rust concepts and expressions are broken down into primitives that can be expressed in Verilog, so that the resulting design can be fed into an existing toolchain.  

To get the synthesizable description of your Circuit, we use the `descriptor`. Here is the relevant part of the `Circuit` trait:

```rust
{{#rustdoc_include ../code/src/circuits/synth.rs:circuit-trait}}
```

Constructing the `Descriptor` can fail at run time as it is possible (and occaisionally useful) to build `Circuit` blocks that are not synthesizable.
Recall from the [Descriptor](circuit_trait/descriptors.md) discussion, that once we obtain a `Descriptor` it has an `hdl` member:

```rust
{{#rustdoc_include ../code/src/circuits/synth.rs:descriptor}}
```

If this field is `Some`, then the underlying `HDLDescriptor` contains a synthesizable translation of the RHDL design into Verilog.  The `HDLDescriptor` is also quite simple:

```rust
{{#rustdoc_include ../code/src/circuits/synth.rs:hdl-descriptor}}
```

You can think of `name` as the "top" element of the circuit.  The `modules` are data structures that define the Verilog code used to describe the circuit.  You can convert these into a pretty printed string using the `.pretty()` method.  The `ModuleList` struct also has a method to check the syntax of the enclosed Verilog using [icarus](https://github.com/steveicarus/iverilog).  So, for a synthesizable circuit `T`, we can do something akin to:

```rust
{{#rustdoc_include ../code/src/circuits/synth.rs:verilog}}
```

For completeness, the resulting Verilog code is the following:

```verilog
{{#include ../code/and_gate.v}}
```

We will cover synthesis and fixturing (where Circuits are connected to outside signals) in detail in a later section.