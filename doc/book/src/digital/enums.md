# Enums

Implementing `Digital` for enums is more complicated than structs, because they include significantly more structure and more information is needed about exactly how the `enum` is represented on the wire.  We include the `prelude` from `rhdl` as usual:

```rust
{{#rustdoc_include digital_ex/src/enum_ex.rs:prelude}}
```

 Let's `impl Digital` for the following (still relatively simple) `enum`:

```rust
{{#rustdoc_include digital_ex/src/enum_ex.rs:def}}
```

First, to determine the number `BITS`, we need to know how many bits in the discriminant.  The minimum number of bits will clearly be 2 (there are 4 possibilities), and then we need to carry up to 16 data bits, so `BITS = 18`.

```rust
{{#rustdoc_include digital_ex/src/enum_ex.rs:BITS}}
```

Because we are `impl Digital` manually, we need to decide on the layout and variant details for our `enum`.  We will put the discriminant in the MSBs, and otherwise, pad the MSBs of the payloads so that they all require the same number of bits.  Our goal is to make something like this:

```badascii
+-+tag+--+-+Arg2++-+Arg1+-+
| 17:16  | 15:8  |  7:0   |
+--------+-------+--------+
| Nop 00 |       |        |
| Add 01 |  .1   |   .0   |
| Sub 10 |  .1   |   .0   |
| Not 11 |       |   .0   |
+--------+-------+--------+
```

Once again, we add the required `Copy, PartialEq, Clone` to our enum:

```rust
{{#rustdoc_include digital_ex/src/enum_ex.rs:derive}}
```

Next, we need to write the `Kind` of this data type, which is the run time type information needed to describe the layout of the enum to RHDL.  Unlike a `struct`, an `enum` requires some more details.  In particular, we need to decide the layout of the discriminant (lsb or msb?  how many bits?  signed, unsigned?).  We will use the following assumption for the discriminant layout:

1. Two bits wide, so that `Noop = 0, Add = 1,` etc
2. Unsigned values
3. MSB aligned, so that the opcode occupies the top 2 bits of the value.

We can know write the `static_kind` method

```rust
{{#rustdoc_include digital_ex/src/enum_ex.rs:static_kind}}
```

As you can see, the code is not significantly more complicated than for a `struct`.  We construct a set of `Variants`, one for each variant, and then populate a top level `Kind::Enum` using the helper functions.

Slightly harder is the `bin` method, which must take into account the padding of the individual fields so that they all occupy the same amount of space (in this case 16 bits).

```rust
{{#rustdoc_include digital_ex/src/enum_ex.rs:bin}}
```

Also, here, the bit vectors are built LSB first (as is always the case in Rust), so we put the payload first, and then the discriminant at the end.  

```admonish warning
Hand coding `static_kind` and `bin` is not a good idea in general.  RHDL assumes that any value that `impl Digital` will serialize bits into `bin` such that the layout is exactly described by `static_kind`.  Breaking this invariant will lead to bad things happening.  Here, we are deriving the trait as an exercise to explain how it works.  In practice, you will want to (almost always) just `#[derive(Digital)]`
```

The last method we need is `dont_care`, which just needs to return _some_ value that is valid for the type.  It doesn't really matter which one, and the `derive` macro shipped with `RHDL` just uses `Default::default()` for the `dont_care` value.  Why isn't it just `default()`?  We need to differentiate between a real `default()` value, which yields a legitimate value, and `dont_care()` which in hardware will create a vector of `x` bits.  RHDL ensures that these bits don't escape your kernel function, so they act much like a `MaybeInit` value.  In any case, to avoid a bunch of `unsafe` everywhere, we require a constructed valid value for `rustc` to use in some instances.

```rust
{{#rustdoc_include digital_ex/src/enum_ex.rs:dont_care}}
```

And that is all it takes to `impl Digital` for an `enum`!  To generate an SVG map of the layout of our `OpCode` enum, we can add the following test function to our crate:

```rust
{{#rustdoc_include digital_ex/src/enum_ex.rs:test_opcode_layout}}
```

![OpCode Layout](digital_ex/opcode.svg)

Note that this agrees well with our hand drawn layout above.
 