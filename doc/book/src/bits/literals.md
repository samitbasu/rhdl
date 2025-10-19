# Literals

Literals are a special case, and worth an aside.  Generally, `rustc` will attempt to infer the type of any integer literal that appears with out a suffix in your code.  For example, if you have an expression like the following,

```rust
let a: b8 = 42.into();
//         👇 - integer literal
let b = a + 1;
```

then `rustc` will attempt to assign a type to the integer literal, and fall back to `i32` if it cannot resolve the type.  For the operators in RHDL, the literal type is matched to the underlying representation, which means `i128` for `SignedBits` and `u128` for `Bits`.  So in the above case, the arguments to the `+` operator are actually `Bits::<8>` and `u128`.  

How does this work with my claim that there is no type coercion in RHDL?  Well, it's more of a form of inference for convenience.  Internally, RHDL will infer the type of the arguments to be the same, and thus the literal will be interpreted as `Bits::<8>`.  In Rust, there are implementations of the various operators that take `u128` in the left and right hand side of the arithmetic expression.  

Not having to include a suffix on every literal is a significant readability improvement over e.g., RustHDL.  But it comes with some caveats.  You may occaisionally need to add a `u128` suffix if `rustc` tries to assume a literal is `i32`, since mixing signed and unsigned types is not allowed in RHDl.  