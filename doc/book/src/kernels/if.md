# If Expressions

RHDL supports the usual syntax for `if` expressions from Rust.  You can use them in the `C` style as statements, or as expressions.  Both are supported.  Here is an `if` as a statement:

```rust
{{#rustdoc_include ../code/src/kernels/if_ex.rs:step_1}}
```

You can simplify this into an expression, and eliminate the binding for `c`, since the function block will take the value of the last expression:

```rust
{{#rustdoc_include ../code/src/kernels/if_ex.rs:step_2}}
```

```admonish note
Just as rust has no ternary operator `?`, RHDL doesn't have one either.  Use an `if` expression, as it is generally easier to read, and less likely to cause confusion.
```

RHDL also supports `if let` (although not chaining).  So you can also use the following:

```rust
{{#rustdoc_include ../code/src/kernels/if_ex.rs:step_3}}
```

Note that RHDL includes some special support for `Option` and `Result`, so there is a bit of extra type inference magic here to make this work.  But it also works with any other `enum` you want to pattern match on:

```rust
{{#rustdoc_include ../code/src/kernels/if_ex.rs:step_4}}
```
