# Late Checking

Dynamic operators are checked late in the compilation process, rather than early.  Consider the following kernel function (I know we haven't covered kernel functions yet, but basically, they are pure Rust functions that are synthesizable...):

```rust
{{#rustdoc_include ../code/src/bits/mod.rs:late_checking_1}}
```

This function `do_stuff` will _compile_ just fine, since `rustc` does not know that `d` is a 6 bit quantity.  However, it is not runnable, and RHDL will reject it.  If we try to evaluate the function for some argument like this:

```rust
{{#rustdoc_include ../code/src/bits/mod.rs:late_checking_1_test}}
```

We get a straight panic:

<!-- cmdrun to-html "cd ../code && cargo test --features doc2 test_run_do_stuff 2>&1" -->

In this case, RHDL's compiler is actually better at spotting exactly where the problem is.  We don't normally need to compile functions manually, but it's simple enough to do in a test case:

```rust
{{#rustdoc_include ../code/src/bits/mod.rs:compile_do_stuff}}
```

<!-- cmdrun to-html "cd ../code && cargo test --features doc3 test_compile_do_stuff -- --nocapture 2>&1" -->

In this case, RHDL has inferred that `d` must be 5 bits wide (based on the conversion to `e`), and thus, the assignment is invalid.
