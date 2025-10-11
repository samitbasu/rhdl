# Enums

Implementing `Digital` for enums is more complicated than structs, because they include significantly more structure and more information is needed about exactly how the `enum` is represented on the wire.  Let's `impl Digital` for the following (still relatively simple) `enum`:

```rust
pub enum OpCode {
    Noop,
    Add(b8,b8),
    Sub(b8,b8),
    Not(b8)
}
```

First, to determine the number `BITS`, we need to know how many bits in the discriminant.  The minimum number of bits will clearly be 2 (there are 4 possibilities), and then we need to carry up to 16 data bits, so `BITS = 18`.


```rust
impl Digital for OpCode {
    const BITS: usize = 18;
}
```

Because we are `impl Digital` manually, we need to decide on the layout and variant details for our `enum`.  We will put the discriminant in the MSBs, and otherwise, pad the MSBs of the payloads so that they all require the same number of bits.  Our goal is to make something like this:

```badascii
+-+bits+-+-+Arg2++-+Arg1+-+
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
#[derive(Copy, PartialEq, Clone)] // 👈 Needed by Digital
pub enum OpCode {
    Noop,
    Add(b8,b8),
    Sub(b8,b8),
    Not(b8)
}
```

Next, we need to write the `Kind` of this data type, which is the run time type information needed to describe the layout of the enum to RHDL.  Unlike a `struct`, an `enum` requires some more details.  In particular, we need to decide the layout of the discriminant (lsb or msb?  how many bits?  signed, unsigned?).  We will use the following assumption for the discriminant layout:

1. Two bits wide, so that `Noop = 0, Add = 1,` etc
2. Unsigned values
3. MSB aligned, so that the opcode occupies the top 2 bits of the value.

We can know write the `static_kind` method

```rust
impl Digital for OpCode {
    fn static_kind() -> Kind {
        let nop_variant = Kind::make_variant("Nop", <() as Digital>::static_kind(), 0);
        let add_variant = Kind::make_variant("Add", <(b8, b8) as Digital>::static_kind(), 1);
        let sub_variant = Kind::make_variant("Sub", <(b8, b8) as Digital>::static_kind(), 2);
        let not_variant = Kind::make_variant("Not", <b8 as Digital>::static_kind(), 3);
        let alignment = rhdl::core::DiscriminantAlignment::Msb;
        let ty = rhdl::core::DiscriminantType::Unsigned;
        let layout = Kind::make_discriminant_layout(2, alignment, ty);
        Kind::make_enum(
            "OpCode",
            [nop_variant, add_variant, sub_variant, not_variant].into(),
            layout,
        )
    }
}
```

As you can see, the code is not significantly more complicated than for a `struct`.  We construct a set of `Variants`, one for each variant, and then populate a top level `Kind::Enum` using the helper functions.

Slightly harder is the `bin` method, which must take into account the padding of the individual fields so that they all occupy the same amount of space (in this case 16 bits).

```rust
impl Digital for OpCode {
    fn bin(self) -> Box<[BitX]> {
        let mut bits = Vec::with_capacity(Self::BITS);
        match self {
            OpCode::Nop => {
                bits.extend((()).bin());
                // 16 bit padding 👇 
                bits.extend(b16(0).bin());
                bits.extend(b2(0b00).bin());
            }
            OpCode::Add(a, b) => {
                bits.extend((a, b).bin());
                bits.extend(b2(0b01).bin());
            }
            OpCode::Sub(a, b) => {
                bits.extend((a, b).bin());
                bits.extend(b2(0b10).bin());
            }
            OpCode::Not(a) => {
                bits.extend(a.bin());
                // 8 bit padding 👇 
                bits.extend(b8(0).bin());
                bits.extend(b2(0b11).bin());
            }
        }
        bits.into_boxed_slice()
    }
}
```

Also, here, the bit vectors are built LSB first (as is always the case in Rust), so we put the payload first, and then the discriminant at the end.  

```admonish warning
Hand coding `static_kind` and `bin` is not a good idea in general.  RHDL assumes that any value that `impl Digital` will serialize bits into `bin` such that the layout is exactly described by `static_kind`.  Breaking this invariant will lead to bad things happening.  Here, we are deriving the trait as an exercise to explain how it works.  In practice, you will want to (almost always) just `#[derive(Digital)]`
```

The last method we need is `dont_care`, which just needs to return _some_ value that is valid for the type.  It doesn't really matter which one, and the `derive` macro shipped with `RHDL` just uses `Default::default()` for the `dont_care` value.  Why isn't it just `default()`?  We need to differentiate between a real `default()` value, which yields a legitimate value, and `dont_care()` which in hardware will create a vector of `x` bits.  RHDL ensures that these bits don't escape your kernel function, so they act much like a `MaybeInit` value.  In any case, to avoid a bunch of `unsafe` everywhere, we require a constructed valid value for `rustc` to use in some instances.

```rust
impl Digital for OpCode {
    fn dont_care() -> Self {
        Self::Nop
    }
}
```

And that is all it takes to `impl Digital` for an `enum`.  Let's add it to our `digital` crate, and get a rendering of the bit layout.  We put our new data structure into it's own module `opcode.rs`

```rust,write:digital/src/opcode.rs
use rhdl::prelude::*;

#[derive(Copy, PartialEq, Clone)]
pub enum OpCode {
    Nop,
    Add(b8, b8),
    Sub(b8, b8),
    Not(b8),
}

impl Digital for OpCode {
    const BITS: usize = 18;
    fn static_kind() -> Kind {
        let nop_variant = Kind::make_variant("Nop", <() as Digital>::static_kind(), 0);
        let add_variant = Kind::make_variant("Add", <(b8, b8) as Digital>::static_kind(), 1);
        let sub_variant = Kind::make_variant("Sub", <(b8, b8) as Digital>::static_kind(), 2);
        let not_variant = Kind::make_variant("Not", <b8 as Digital>::static_kind(), 3);
        let alignment = rhdl::core::DiscriminantAlignment::Msb;
        let ty = rhdl::core::DiscriminantType::Unsigned;
        let layout = Kind::make_discriminant_layout(2, alignment, ty);
        Kind::make_enum(
            "OpCode",
            [nop_variant, add_variant, sub_variant, not_variant].into(),
            layout,
        )
    }
    fn bin(self) -> Box<[BitX]> {
        let mut bits = Vec::with_capacity(Self::BITS);
        match self {
            OpCode::Nop => {
                bits.extend((()).bin());
                bits.extend(b16(0).bin());
                bits.extend(b2(0b00).bin());
            }
            OpCode::Add(a, b) => {
                bits.extend((a, b).bin());
                bits.extend(b2(0b01).bin());
            }
            OpCode::Sub(a, b) => {
                bits.extend((a, b).bin());
                bits.extend(b2(0b10).bin());
            }
            OpCode::Not(a) => {
                bits.extend(a.bin());
                bits.extend(b8(0).bin());
                bits.extend(b2(0b11).bin());
            }
        }
        bits.into_boxed_slice()
    }
    fn dont_care() -> Self {
        Self::Nop
    }
}
```

and then link this into the `lib.rs`

```rust,write:digital/src/lib.rs
pub mod things;
pub mod opcode; // 👈 New!
```

We can check that this compiles

```shell,rhdl:digital
cargo check -q
```

Now let's add a simple test function that generates an SVG map of our `Things` data structure.  It is available on any `Kind`.

```rust,write:digital/tests/make_opcode_svg.rs
use rhdl::prelude::*;
use digital::*;

#[test]
fn test_things_svg() {
    let svg = opcode::OpCode::static_kind().svg("OpCode");
    std::fs::write("opcode.svg", svg.to_string()).unwrap();
}
```

```shell,rhdl:digital
cargo build -q
cargo nextest run
```

```shell,rhdl-silent:digital
cp opcode.svg $ROOT_DIR/src/img/.
```

The result is the following handy SVG map of the layout of our type:

![Things SVG](../img/opcode.svg)

Note that this agrees well with our hand drawn layout above.
