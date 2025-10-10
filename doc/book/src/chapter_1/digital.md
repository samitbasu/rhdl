# Digital

After the `Bits` type, the next most critical trait is `Digital`.  Here are the critical elements of the `Digital` trait that must be provided for any type.  I've redacted those methods that are automatically provided by the trait itself so we can focus on the important ones:

```rust
pub trait Digital: Copy + PartialEq + Sized + Clone + 'static {
    /// Associated constant that gives the total number of bits needed to represent the value.
    const BITS: usize;
    /// Returns the [Kind] (run time type descriptor) of the value as a static method
    fn static_kind() -> Kind;
    /// Returns the binary representation of the value as a vector of [BitX].
    fn bin(self) -> Box<[BitX]>;
    /// Returns a "don't care" value for the type.
    fn dont_care() -> Self;
}
```

Most of these are fairly easy to understand.  A `Digital` value has a total of 3 representations:

1. The internal Rust representation used bu `rustc` and in your code when it's running.
2. A bit-wise representation that may include `dont-care` bits (usually marked with an `x`) for hardware description and synthesis.
3. A trace representation that additionally include high-impedance states (usually marked with a `z`) and don't care states.  For almost all types, the trace representation can be inferred from the bit-wise representation.

The `dont-care` bits are important, particularly for Rust enums.  But in general, you can create any data structure you want, and `impl Digital` for it, and it will be supported by RHDL, which is pretty awesome.  

```admonish note
An `impl Digital` type may have *ZERO* bits!  This is really quite important, as a lot of idiomatic Rust code relies on the availability of Zero Sized Types (ZSTs), and these are also allowed in RHDL.  All ZSTs will be removed before a design is sent for hardware synthesis.
```

For the vast majority of cases, though, you will probably build data structures by composition of existing atomic types (like `Bits`, `bool`, `SignedBits`).  For example, these all `impl Digital`:

```rust
b4        // - a 4 bit vector
()        // - a ZST
(b4, b5)  // - a tuple of bit vectors
[b4; 8]   // - an array of bit vectors
u8        // - the core Rust integer types, like i8,u8,i16,etc.
```

From these base types, you can construct more complicated types using structural composition.  But it's much easier to use `struct` and `enum` instead.  

## Digital Struct

It's fairly easy to understand `impl Digital` for structs.  The following struct, for example:

```rust
pub struct Things {
    pub count: b4,
    pub valid: true,
    pub coordinates: (i6, i4),
    pub zst: (),
}
```

You can `impl Digital` for this type yourself.  It's not difficult, and serves as a good exercise in understanding how little magic there is in the macro that does it for you.  For the number of `BITS`, we have `4 + 1 + 10 + 0 = 15`, so this should be a 15 bit type.

```admonish note
The number of `BITS` used by a `Digital` value is generally unrelated to the `rustc` representation.  A `bool`, for example, only occupies 1 bit in RHDL, not 8.  And a `Bits<N>` bit vector occupies `N` bits.
```

```rust
impl Digital for Things {
    const BITS: usize = 15;
}
```

We see that `Digital: Copy + PartialEq + Clone` (the other requirements are less interesting like `Sized` and `'static`.). So we add these to our `Things` declaration.

```rust
// 👇 - new
#[derive(Copy, PartialEq, Clone)]
pub struct Things {
    pub count: b4,
    pub valid: bool,
    pub coordinates: (s6, s4),
    pub zst: (),
}
```

The next thing we need is the `Kind` of this data type.  You can think of this as the run time type information needed to describe the shape of the datastructure to RHDL.  It is completely different than `rustc`s in-memory layout in several ways.  For example, `rustc` will use 8 bits for a boolean, we will use 1 bit.  `rustc` will use 128 bits for each of the `b4, i6, i4` values (since that is part of their generic representation).  So we need a layout that describes exactly how the `Things` struct is laid out in those 15 bits.  In RHDL, this is captured in the `Kind` type, and you can read the docs for it directly.  We won't go through those docs in detail, but here is what the implementation of `static_kind` might look like:

```rust
fn static_kind() -> Kind {
    let count_field = Kind::make_field("count", <b4 as Digital>::static_kind());
    let valid_field = Kind::make_field("valid", bool::static_kind());
    let coordinates_field =
        Kind::make_field("coordinates", <(s6, s4) as Digital>::static_kind());
    let zst_field = Kind::make_field("zst", <() as Digital>::static_kind());
    Kind::make_struct(
        "Things",
        [count_field, valid_field, coordinates_field, zst_field].into(),
    )
}
```

As you can see, it simply delegates to the `static_kind` of each of the fields, enumerates them in the order they appear in the struct, and adds names.   We need only provide two additional methods.  The first packs the data into a bit vector.  We can delegate similarly to the individual fields, and pack them in source-code order (i.e., first field listed occupies the LSBs of the bit vector):

```rust
fn bin(self) -> Box<[BitX]> {
    let mut bits = Vec::with_capacity(Self::BITS);
    bits.extend(self.count.bin());
    bits.extend(self.valid.bin());
    bits.extend(self.coordinates.bin());
    bits.extend(self.zst.bin());
    bits.into_boxed_slice()
}
```

Similarly, the `dont_care` method can be delegated as well:

```rust
fn dont_care() -> Self {
    Self {
        count: b4::dont_care(),
        valid: bool::dont_care(),
        coordinates: <(s6, s4)>::dont_care(),
        zst: (),
    }
}
```

And that is all!  Let's check our definition of our new type.  We can visualize the `Kind` of any `impl Digital` type by rendering it to a handy SVG.  

```shell,rhdl-silent
rm -rf digital
```

Let's start by creating a new Rust library to hold our various experiments.

```shell,rhdl
cargo new --lib digital
cd digital
cargo add --path ~samitbasu/Devel/rhdl/crates/rhdl
cargo add --dev miette
```

We will place our new data structure into it's own module `things.rs`:

```rust,write:digital/src/things.rs
use rhdl::prelude::*;

#[derive(Copy, PartialEq, Clone)]
pub struct Things {
    pub count: b4,
    pub valid: bool,
    pub coordinates: (s6, s4),
    pub zst: (),
}

impl Digital for Things {
    const BITS: usize = 15;
    fn static_kind() -> Kind {
        let count_field = Kind::make_field("count", <b4 as Digital>::static_kind());
        let valid_field = Kind::make_field("valid", bool::static_kind());
        let coordinates_field =
            Kind::make_field("coordinates", <(s6, s4) as Digital>::static_kind());
        let zst_field = Kind::make_field("zst", <() as Digital>::static_kind());
        Kind::make_struct(
            "Things",
            [count_field, valid_field, coordinates_field, zst_field].into(),
        )
    }
    fn bin(self) -> Box<[BitX]> {
        let mut bits = Vec::with_capacity(Self::BITS);
        bits.extend(self.count.bin());
        bits.extend(self.valid.bin());
        bits.extend(self.coordinates.bin());
        bits.extend(self.zst.bin());
        bits.into_boxed_slice()
    }
    fn dont_care() -> Self {
        Self {
            count: b4::dont_care(),
            valid: bool::dont_care(),
            coordinates: <(s6, s4)>::dont_care(),
            zst: (),
        }
    }
}
```

and then link this into the `lib.rs`

```rust,write:digital/src/lib.rs
pub mod things;
```

We can check that this compiles

```shell,rhdl:digital
cargo check -q
```

Now let's add a simple test function that generates an SVG map of our `Things` data structure.  It is available on any `Kind`.

```shell,rhdl:digital
mkdir tests
```

```rust,write:digital/tests/make_svg.rs
use rhdl::prelude::*;
use digital::*;

#[test]
fn test_things_svg() {
    let svg = things::Things::static_kind().svg("Things");
    std::fs::write("things.svg", svg.to_string()).unwrap();
}
```

```shell,rhdl:digital
cargo build -q
cargo nextest run
```

```shell,rhdl-silent:digital
cp things.svg $ROOT_DIR/src/img/.
```

The result is the following handy SVG map of the layout of our type:

![Things SVG](../img/things.svg)

You can now see the precise layout of the fields in the struct, as well as the elimination of the ZST field.