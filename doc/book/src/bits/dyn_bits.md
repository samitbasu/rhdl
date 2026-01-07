# Dynamic Width Bits

In general to work with dynamic width bitvectors, you will do the following:

1. Start with statically sized bitvectors `Bits::<N>`.
2. Convert them to dynamic bitvectors using `.dyn_bits()` or using the extension operators, like `xadd`, `xmul`, etc.
3. When the calculations are complete, convert the `DynBits` back to a compile-time sized `Bits::<N>` by casting it with `.as_bits()`.  This call is generic over the number of bits in the output, but will panic if there is a mismatch between the runtime size, and the compile time size of the destination.

An example is simpler to understand.  

```rust
{{#rustdoc_include bits_ex/src/main.rs:dyn-bits-ex}}
```

While this example is not particularly interesting, it becomes much more difficult when the intermediate operations include bit shifting, sign conversion, etc.  For a more realistic example, consider linear interpolation.  We have two values `a: b8`, and `b: b8`, and an interpolant `x: b4`.  We want to compute as accurately as possible, the expression:

```rust
 c = (a * x + b * (16 - x)) >> 4 
```

This quickly becomes complicated, as the intermediate expressions need to have enough bits to store the products of `b8 x b4 -> b12`, but then a subtraction of a 5 bit literal and a 4 bit unsigned value, needs 6 bits for storage, etc.  However, the end result is still an 8 bit value, and so our function implementation looks like this:

```rust
{{#rustdoc_include bits_ex/src/main.rs:lerp-ex}}
```

we can use dynamic bit widths internally to get fine grained control over which bits we keep and which we throw away.  And while in this case, you could hard code all of the intermediate bit widths, it becomes much more convenient when `lerp` is generic over the input and output bit widths.

Note that simply promoting an unsigned `Bits<N>` to a `SignedDynBits` will require an extra bit.  This is because the range of an unsigned `N` bit value is `0..2^N-1`.  To store a positive value of size `2^N-1` requires `N+1` bits in a signed 2's complement integer.  Hence the need for an extra bit.

The process for `SignedBits` is entirely analogous.  A `SignedBits<N>` typed value can be converted to a `SignedDynBits` value using the `.dyn_bits()` method.  This bit vector will have it's size erased from the type signature, and will track the bit width at runtime.  You can then operate on the `SignedDynBits` using either the bit-width preserving operators (like `+`, `-`), which will wrap out of range results, or using the extended operators (like `xadd, xmul, xext`) which will preserve bits, but require more/different output bit widths.  When ready, you can then convert the `SignedDynBits` value back to a `SignedBits` of the correct width.

```admonish note
The goal of `DynBits` is not to have RHDL do a bunch of magic extra bit manipulation for you.  Instead it is to enable you to have precise control over how the bits are manipulated and where they are dropped or preserved.
```
