# Fixtures

The `Fixture` is an opaque struct, and the internal details are subject to change:

```rust
{{#rustdoc_include ../code/src/fixturing.rs:fixture-def}}
```

The circuit is a type that `impl Circuit`, and each of the drivers for the circuit are generic over this same type.  A simplified summary of the `Fixture` API is provided here:

```rust
{{#rustdoc_include ../code/src/fixturing.rs:impl-fixture}}
```

Each of these methods is described in more detail in the following sections.

