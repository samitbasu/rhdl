# Construction

To construct the fixture, we provide the name `name` of the fixture (used in the Verilog module name as the top level module) and an _instance_ of a circuit `t` that implements the `Circuit` trait.  Note that the circuit is not a type parameter, but an actual instance of the circuit.  This is because the fixture is generated for a particular instance of the circuit (configured and initialized as required).   

To illustrate, let's consider a simple AND gate circuit (the kernel has been omitted for brevity):

```rust
{{#rustdoc_include ../code/src/fixturing.rs:AND-gate}}
```

And now suppose we attempt to build a fixture for this gate and export it:

```rust
{{#rustdoc_include ../code/src/fixturing.rs:empty-fixture}}
```

We get the following error:

<!-- cmdrun to-html "cd ../code && cargo test --lib -- fixturing::fixture_new::test_fixture_and --exact --nocapture --ignored 2>&1" -->

This is because the `ANDGate` has 2 input signals, and our fixture has not connected them to anything at the top level.

```admonish, warning
You must provide an input for all circuit inputs when building a fixture.  Failing to do so will result in an error.  You can leave outputs unconnected, but all inputs must be connected.
```
