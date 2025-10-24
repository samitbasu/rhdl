# Aliases

To save on typing and to make your code easier to read, there are a set of type aliases that can be used in your code.  For regular (non-dynamic) bit vectors, the aliases take the form of `b1` through `b128`, and each represents a _type_ that can hold `N` bits.  Thus, you can declare a 13 bit vector in your rust code as:

```rust
let a: b13;
```

The name was chosen to suggest "bits", and to not conflict with `u8, u16, etc` from `rustc`'s standard types.  

Similarly for signed bitvectors, you can declare a 12 bit signed bit vector in your code as:

```rust
let c: s12
```

There is no `s1` alias, as it doesn't really make sense to have a 1-bit signed integer. 

Remember that these are just `type` definitions, and you do not have to use them.  You can just as simply write:

```rust
let a: Bits::<13>;
let c: SignedBits::<12>;
```

The shorter version tends to make code more readable, as the heavier notation for types doesn't convey much unique information in code that uses a lot of bitvectors.   But the choice is up to you.  If you end up needing to write generic code, like a function that does something like:

```rust
fn my_func<const N: usize>(a: Bits::<N>) -> Bits::<N>  where rhdl::bits::W<N>: BitWidth {
   let c: SignedBits::<N>;
   // Compute fancy stuff!
   a
}
```

then the use of the type notation is required.

