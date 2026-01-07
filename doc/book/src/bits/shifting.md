# Shifting

There are only 2 shifting operators in RHDL.  These are the wrapping left shift `<<` and the wrapping right shift `>>` operators.  Here, `wrapping` does not mean that bits "wrap around".  They simply "fall off" the end.  The `wrapping` refers to the fact that you cannot overflow or underflow a value by shifting it.  

The left shift operation is the same for `Bits` and `SignedBits` - it simply inserts zeros from the LSB:

```rust
{{#rustdoc_include bits_ex/src/main.rs:left_shift}}
```

For a signed value, when the MSB changes due to the shift, the sign of the value will change.  Again, this is normal for 2's complement arithmetic and is implemented in Rust for `wrapping` shifts.

```rust
{{#rustdoc_include bits_ex/src/main.rs:signed_left_shift}}
```

In the last step, `a` went from a large positive number to a large negative number.

```admonish warning
There are a few critical differences between how `rustc` handles integers, and how RHDL handles integers.  One is that for `rustc` shifting by more than `N` bits (where `N` is the bitwidth of the integer) does nothing.  In RHDL, it will generally produce `0`.
```

For right shift operations, the behavior is different for `Bits` than for `SignedBits`.  For `Bits`, right shifting simply injects zeros at the MSB of the vector, so that all the bits shift right.

```rust
{{#rustdoc_include bits_ex/src/main.rs:right_shift}}
```

For a `SignedBits` value, the sign bit is shifted in from the MSB, so that negative values remain negative.

```rust
{{#rustdoc_include bits_ex/src/main.rs:signed_right_shift}}
```

For both left and right shift operators, you can use a bit vector `Bits` to specify the shift, so this will also work:

```rust
{{#rustdoc_include bits_ex/src/main.rs:bit_bit_shift}}
```

The shift amount cannot be signed.

```admonish warning
Shift operators (when implemented in hardware) can either be very cheap or very expensive.  If RHDL can work out the number of bits to be shifted at compile time, then a shift operator is trivial.  It can be implemented by simpling reassigning the bits and discarding the others.  However, if the shift amount cannot be determined at compile time, but can vary at runtime, then the resulting hardware implementation can be quite large and complicated.  Search for "Barrel Shifters" for more details on how these are typically implemented.
```
