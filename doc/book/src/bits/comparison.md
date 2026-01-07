# Comparison Operators

The comparison operators `>,>=,==,!=,<=,<` operate must the same was as one would expect from `rustc`.  The only catch is that comparison operators do not return `b1`, they return a `bool`, which is required by the Rust language specification.  So for example:

```rust
{{#rustdoc_include bits_ex/src/main.rs:comparison-ops}}
```

Generally, prefer to use `bool` instead of `b1` whenever possible in your design.  It will be easier to use the value in a conditional context if ultimately you need to branch depending on the value.  Comparison operators (apart from `==` and `!=`) are different for signed and unsigned bit vectors.  RHDL will generate hardware descriptions for the comparison operators that includes the appropriate sign handling if the operands are signed.

```admonish note
Unlike some HDLs, RHDL does not allow you compare values of different types with the same exception for literals as before.  You can compare a bitvector `Bits::<N>` with a literal, and both `rustc` and `RHDL` will promote the literal to the equivalent bitvector length.  
```
