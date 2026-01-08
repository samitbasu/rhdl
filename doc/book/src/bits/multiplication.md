# Multiplication

Multiplication is in some ways, similar to any other binary op:

1. The two arguments to the multiplication operator must be the same type (unless one is a literal, in which case it is promoted).
2. If the two arguments are signed, the output is the signed product.  Otherwise it is the unsigned product.
3. The product is wrapped to fit in the number of bits, so the product of an `N` bit value and a second `N` bit value will occupy `N` bits.  This is identical to how `rustc` handles multiplication of integers, except that no panic will occur if the value overflows the output size.

The primary distinction about multiplication is that in general, it can be quite resource intensive to synthesize.  And so you generally need to be careful about introducing multipliers into your hardware designs.  Of course, on some devices, dedicated hardware multipliers may exist, or other dedicated circuitry (such as a Digital Signal Processing or DSP block) that can perform multiplies efficiently. Unfortunately, RHDL does not know about these features.  So it allows you to write multiply operations, and then relies on your toolchain to handle them.

```rust
{{#rustdoc_include ../code/src/bits/mod.rs:mul_unsigned}}
```

The signed case is similar:

```rust
{{#rustdoc_include ../code/src/bits/mod.rs:mul_signed}}
```

