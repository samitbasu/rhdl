# Early returns and Try expressions

RHDL supports both early returns and `try` expressions.  To make this work with late type inference is a bit tricky, and you may find that in some instances, you need to add type signatures to help RHDL understand the types involved, even though `rustc` is able to figure them out.  First off, you can early return from a kernel function, as you would expect:

```rust
{{#rustdoc_include ../code/src/kernels/early.rs:step_1}}
```

RHDL will add the needed control flow and placeholders to shortcut the remainder of the kernel when an early return is encountered.  The second way to early return is with a `try` expression.  For the most part, these also function the way you would expect.  You can have a `kernel` that returns a `Result<O, E>`, provided `O: Digital` _and_ `E: Digital`.  You can also have a kernel that returns a `Option<T>` where `T: Digital`.  In both cases, if you apply `?` to an expression, if can early return, just as in normal Rust.  This feature is quite handy for adding non-trivial error handling to your hardware designs.  Coupled with `enum`, you can make your code far clearer and error-explicit.  Here is an example with `Result`:

```rust
{{#rustdoc_include ../code/src/kernels/early.rs:step_2}}
```

Nice, right?  Here is an example with Option, which might be better for instances in which failures should be silently discarded.

```rust
{{#rustdoc_include ../code/src/kernels/early.rs:step_3}}
```

```admonish note
Using `Option` in hardware designs is a good pattern. Normally, one has some kind of data lines, and then a `valid` strobe that indicates that there is valid data on the lines.  It is your problem to ensure that you read the data lines only when the `valid` strobe is asserted.  You can change this into an `Option<T>`, in which case RHDL will ensure that you can only see the data when the contents are `Some`.  Thus guaranteeing that the invariant promised by the protocol (only look at data when valid is `true`) is upheld by the compiler in accordance with the type system.
```
