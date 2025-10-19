# Constructor Functions

There are several ways to construct `Bits` values (and `SignedBits` values as well).  The first is to use `.into()`, such as:

```rust
let a: b8 = 42.into()
```

Unfortunately, this is not (currently) synthesizable.  So you cannot use it in functions marked with `#[kernel]`.  However, there are helper constructors for each bitwidth, which happen to have the same names as the types `b1..b128`.  So you could write this expression as:

```rust
let a = b8(42)
```

In this case, the type of `a` is inferred from the return type of `b8`, which is `Bits::<8>` or equivalently `b8`.  There is also a `bits` function, which is generic over the number of bits, and which can be used in places where `rustc` is able to infer the number of bits required.  So you can write, for example:

```rust
let a: b8 = bits(42)
```

This form is handy when writing code that is generic over a bitwidth, as you can either constrain the variable or be explicit with the `bits` invokation using the turbofish syntax:

```rust
let a = bits::<N>(42); // Where const N: usize, and W<N>: BitWidth
```

For signed bit values, the situation is slightly more complicated.  The `.into` still works in regular Rust code (non-synthesizable), provided you put the negative sign inside parentheses.  Otherwise `rustc` gets confused as to the type requested:

```rust
let a: s8 = (-42).into(); // Works!  But is not synthesizable....
let a: s8 = -42.into();   // Doesn't work!
```

Analogous to the `b1..b128` constructor functions, there are `s2..s128` signed constructor fucntions that take an `i128` argument and return a signed bit vector of the requested length.  

```admonish warning
The constructor functions will panic if you attempt to construct a value with an out-of-range literal!  This is to prevent you from operating under the assumption that the literal you specified was correctly represented in the target type.
```

```rust
let a = s8(-42);  // Works and is synthesizable
let a: s8 = signed(-42); // Works and is also synthesizable
let a = signed::<8>(-42); // Works and is synthesizable
```

