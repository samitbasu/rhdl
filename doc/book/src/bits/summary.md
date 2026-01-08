# Bits

Hardware design tends to involve the manipulation of various numbers of bits, not just the usual `8, 16, 32, 64, 128` that one deals with in normal software.  For example, if we have a physical connection to a device that contains 4 LEDs, we will need to provide exactly `4` bits to indicate which LED should be lit.  To that end, RHDL provides a set of types to model different bit widths.  The base type of interest is `Bits<>` where `U: BitWidth`.  While ideally, `U` would be a const-generic (like `Bits<const N: usize>`) that really doesn't work well enough in stable for what we need.  So RHDL includes a set of typenums `U1, 2,... U128` to indicate the width of the bit vector.  You cannot construct a single bitvector that is larger than 128 bits currently, but of course, you can concatenate multiple bitvectors into a single data structure to create bit vectors of arbitrary (but finite) size.

The documentation of `rhdl-bits` is fairly extensive, but basically, the following is the short-intro.

- There is a `Copy` type that is generic over the bit width, and can hold up to 128 bits.  It is called `Bits<>`.
- There are aliased types for each size from 1 to 128 bits.  These types are called `b1, b2, ..., b128`, and are simply aliases like: `type b2 = Bits<2>`.
- There are constructor functions also called `b1, b2, ..., b128` that allow you to make `Bits<>` from a literal `u128` value, and will panic if you provide an out-of-range value.

```admonish warning
The `rhdl-bits` crate is not meant to provide a general bit-width integer type for use in your Rust applications.  There are several better alternatives for that available on [crates.io](https://crates.io).  Instead, this crate is meant to provide types that behave the same way as hardware fixed-width integers.
```

All of these are basically illustrated in the following short code snippet:

```rust
{{#rustdoc_include ../code/src/bits/mod.rs:basic_usage}}
```

- There is a `Copy` type that is generic over the bit width, and can hold up to a 128 bit _signed_ integer.  It is called `SignedBits<>`.
- There are aliased types for each size of signed bits from 1 to 128 bit wide.  These are called `s1, s2, ..., s128`, and are simply aliases like: `type s2 = SignedBits<2>`.
- There are constructor functions also called `s1, s2, ..., s128` that allow you to make `SignedBits<>` from a literal `i128` value and will panic if you provide an out-of-range value.

```rust
{{#rustdoc_include ../code/src/bits/mod.rs:basic_signed_usage}}
```

Roughly speaking the type `b4` tries to work as much like `u8` as possible, but with the following restrictions:

- You cannot use `as`, since that only works for primitive types.  So you cannot, for example say `12 as b4`.  But you can do `b4(12)`.
- All bit vectors implement wrapping arithmetic _only_.  So, `a+b` where `a: b4` and `b:b4` is equivalent to `a.wrapping_add(b)`.  No arithmetic operation on `Bits<_>` can panic.
- All arithmetic on `Bits` and `SignedBits` is 2's complement.  This is basically the same as `Wrapping`, but without the explicit `Wrapping(_)` wrapper.

You _cannot_ freely cast from one bitwidth to another.  So this won't work in RHDL:

```rust
{{#rustdoc_include ../code/src/bits/mod.rs:failed_bitcast}}
```

Attempting to compile this will cause an error:

<!-- cmdrun to-html "cd ../code && cargo test --features doc4 2>&1" -->


This may come as a bit of a shock if you are used to the permissive casting allowed in other HDLs like Verilog, but in Rust, you can't do this with "normal" integers, and in RHDL, you cannot either.  In general, if you find yourself needing to cast (and there are explicit cast operations), you may have the wrong data structure in hand.  More on that later.

You also cannot index into a bitvector arbitrarily either.  So the expression `a[i]` makes no sense if `a: b4` any more than it would if `a: u8`.  If you need an array of individually mutable bits, there are other ways to express those, such as `[bool; 4]` or `[b1; 4]`.

With the exception of division, most other operations are available on bit vectors, including addition, subtraction, multiplication, etc.  You can checkout the docs for more details.

Finally, note that `b1` and `bool` are not the same thing.  a `b1` is a 1-bit unsigned integer that can either be `1` or `0`, while a `bool` can either be `true` or `false`.  You may use `bool` in your design (it `impl Digital`), but it is not interchangable with `b1`.  In particular, Rust expressions that require a `bool` will not accept a `b1`:

```rust
{{#rustdoc_include ../code/src/bits/mod.rs:bool_not_b1}}
```

with error:

<!-- cmdrun to-html "cd ../code && cargo test --features doc5 2>&1" -->