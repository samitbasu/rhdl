# Aliases

To save on typing and to make your code easier to read, there are a set of type aliases that can be used in your code.  For regular (non-dynamic) bit vectors, the aliases take the form of `b1` through `b128`, and each represents a _type_ that can hold `N` bits.  Thus, you can declare a 13 bit vector in your rust code as:

```rust
{{#rustdoc_include bits_ex/src/main.rs:alias-b13}}
```

The name was chosen to suggest "bits", and to not conflict with `u8, u16, etc` from `rustc`'s standard types.  

Similarly for signed bitvectors, you can declare a 12 bit signed bit vector in your code as:

```rust
{{#rustdoc_include bits_ex/src/main.rs:alias-s12}}
```

There is no `s1` alias, as it doesn't really make sense to have a 1-bit signed integer. 

Remember that these are just `type` definitions, and you do not have to use them.  You can just as simply write:

```rust
{{#rustdoc_include bits_ex/src/main.rs:explicit_versions}}
```

The shorter version tends to make code more readable, as the heavier notation for types doesn't convey much unique information in code that uses a lot of bitvectors.   But the choice is up to you.  If you end up needing to write generic code, like a function that does something like:

```rust
{{#rustdoc_include bits_ex/src/main.rs:where-for-fn}}
```

then the explicit notation is required. 
