# Bitwise Logical Operators

The bitwise logical operators provided by RHDL are the same as those provided by `rustc`: OR, AND, XOR, and NOT.  So you can freely do all of the following:

```rust
{{#rustdoc_include ../code/src/bits/mod.rs:logical}}
```

The bitwise logical operators are also provided for signed values, although they make a lot less sense there.  But if for some reason you need to bit-manipulate a signed value, you can use all the same logical operators.

There are 3 reduction operators that are handy to use in hardware designs, these are ANY, ALL and XOR.  They are methods on both `Bits` and `SignedBits`.  They function as:

```rust
{{#rustdoc_include ../code/src/bits/mod.rs:reduction}}
```

They are all synthesizable.
