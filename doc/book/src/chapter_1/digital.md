# Digital

After the `Bits` type, the next most critical trait is `Digital`.  Here are the critical elements of the `Digital` trait that must be provided for any type.  I've redacted those methods that are automatically provided by the trait itself so we can focus on the important ones:

```rust
pub trait Digital: Copy + PartialEq + Sized + Clone + 'static {
    /// Associated constant that gives the total number of bits needed to represent the value.
    const BITS: usize;
    /// Returns the [Kind] (run time type descriptor) of the value as a static method
    fn static_kind() -> Kind;
    /// Returns the binary representation of the value as a vector of [BitX].
    fn bin(self) -> Vec<BitX>;
    /// Returns a "don't care" value for the type.
    fn dont_care() -> Self;
}
```

Most of these are fairly easy to understand.  A `Digital` value has a total of 3 representations:

1. The internal Rust representation used bu `rustc` and in your code when it's running.
2. A bit-wise representation that may include `dont-care` bits (usually marked with an `x`) for hardware description and synthesis.
3. A trace representation that additionally include high-impedance states (usually marked with a `z`) and don't care states.

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

As you can see, it simply delegates to the `static_kind` of each of the fields, enumerates them in the order they appear in the struct, and adds names.   

We need only provide 