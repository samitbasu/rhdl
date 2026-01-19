# Tuples

Tuples are well supported in RHDL, and there are generic impls that mean any tuple of `Digital` elements is itself `Digital` (up to some length).  Tuples can be formed and deconstructed at will within your code.  There is not much more to say.  Here is a simple example:

```rust
{{#rustdoc_include ../code/src/kernels/tuples.rs:step_1}}
```

