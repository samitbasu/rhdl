# Digital Types


After the `Bits` type, the next most critical trait is `Digital`.  Here are the critical elements of the `Digital` trait that must be provided for any type.  I've redacted those methods that are automatically provided by the trait itself so we can focus on the important ones:

```rust
{{#rustdoc_include digital_ex/src/summary_ex.rs:digital-trait}}
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

| Expression | Description |
|------------|-------------|
| `b4`        | 4 bit vector |
| `()`        | ZST |
| `(b4, b5)`  | tuple of bit vectors |
| `[b4; 8]`   | array of bit vectors |
| `u8`        | core Rust integer types like `i8`, `u8`, `i16`, etc. |

From these base types, you can construct more complicated types using structural composition.  But it's much easier to use `struct` and `enum` instead.  

```admonish note
I want to walk you through the process of `impl Digital` manually before introducing the `#[derive(Digital)]` macro, so you understand what is happening under the hood.  In practice, the macro is much easier and less error prone.
```
