## Basic Testing

We probably want to test all possible inputs of our `XorGate`, and since there are only four inputs, it shouldn't be too hard.  We can start by testing our kernel itself.  Just as a plain Rust function (which it still is...)

```rust
{{#rustdoc_include ../code/src/xor.rs:xor-step-10}}
```

<!-- cmdrun to-html "cd ../code && cargo test --package code --lib -- xor::step_8::test_all_inputs --exact --nocapture" -->

Ok - that was easy enough.  But that just tests that our logic was correct, right?  What about testing more of the things?  How do I know the generated hardware will work as intended?  And what does the generated hardware look like, anyway?  The simpleset way to get a view on the generated HDL is to use the `.hdl` method on any struct that `impl Circuit`.  The result can be converted into a module and then a string.   The following test does exactly that.

```rust
{{#rustdoc_include ../code/src/xor.rs:xor-step-11}}
```

The resulting Verilog looks like this:

```verilog
{{#include ../code/xor_gate.v}}
```

While not required, it is often handy to check that the output of an HDL generation step has not changed from the last time you reviewed or tested it.  As such, a crate like [expect-test](https://github.com/rust-analyzer/expect-test) can be used to check that the output is still correct. 

