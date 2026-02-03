# One Counter

The implementation of the Ones Counter has a few interesting features on its own.

We repeat the setup steps we had previously:

```shell
cargo new --lib ones
cd ones
cargo add rhdl rhdl-toolchains
cargo add --dev miette --features fancy
```

Here is the body of the ones counter, partially built:

```rust
{{#rustdoc_include ../code/src/count_ones.rs:ones-step-1}}
```

The kernel is the function of interest.  For now, we put a placeholder `todo!()` in the body.

```rust
{{#rustdoc_include ../code/src/count_ones.rs:ones-step-2}}
```

For the kernel itself, we need to actually count the ones in `b8`.  While the `Bits` type doesn't have a built in `count_ones` method, you can easily just count the ones in a `for` loop.  For loops are allowed in synthesizable code under fairly constrained circumstances.  RHDL must be able to unroll the loop at compile time, so the indices must be computable at compile time.  This generally means either literals or simple expressions of `const` parameters.   

Also, we can use a mutable local variable to hold the count.  This is totally fine, as mutable local variables are supported by RHDL.  The end result is a kernel function that looks like this:

```rust
{{#rustdoc_include ../code/src/count_ones.rs:ones-step-3}}
```

You can see the logic pretty clearly (which is the point!).  We start with a nibble counter set to zero, and then test the bits one at a time.  This is a "naive" implementation, as it creates a really long chain of adders, but for now, let's leave that alone.  We can improve it if/when needed.  The completed `lib.rs` looks like this:

```rust
{{#rustdoc_include ../code/src/count_ones.rs:ones-step-4}}
```

Let's move to testing.