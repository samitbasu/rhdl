# Literals

RHDL supports only two types of literals:

- Integer literals with the following prefixes:
    - `0x` for hexadecimal literals
    - `0b` for binary literals
    - `0o` for octal literals
    - unprefixed for decimal literals
- Boolean literals `true/false`

These are the normal rules for Rust as well.  The remaining literal types (strings, byte strings, characters, etc) are not supported.  You can have byte strings and characters, but you will need to represent them as synthesizable constructs (usually 8-bit ASCII values) and manipulate them as integers.

You can also segment your literals using the `_` spacer to make them more readable, and the literals are case insensitive.  Note that a bare constant like `42` is not synthesizable, since it doesn't have a well defined length.  But where the length can be inferred from context, you can use a bare constant.  So in this kernel, all literals are passed into the `b8` constructor function:

```rust
{{#rustdoc_include ../code/src/kernels/literals.rs:step_1}}
```

In this example, we use a bare literal, and RHDL determines that it must be an 8-bit literal, as it is being added to an 8-bit literal:

```rust
{{#rustdoc_include ../code/src/kernels/literals.rs:step_2}}
```

If RHDL infers a size for your literal that won't hold the value you specify, you will get an error during the RHDL compilation of your kernel function, or a run time panic if you try to exercise the `kernel` manually.

```rust
{{#rustdoc_include ../code/src/kernels/literals.rs:step_3}}
```

Here is an example of a runtime panic when the function is called

```rust
{{#rustdoc_include ../code/src/kernels/literals.rs:step_3_runtime}}
```

<!-- cmdrun to-html "cd ../code && cargo test --lib -- kernels::literals::step_3::test_panic_at_runtime --exact --nocapture --ignored 2>&1" -->

And here, we try tp compile it using RHDL's `compile_design` function, which results in a compilation error from the RHDL compiler:

```rust
{{#rustdoc_include ../code/src/kernels/literals.rs:step_3_compile}}
```

<!-- cmdrun to-html "cd ../code && cargo test --lib -- kernels::literals::step_3::test_fail_at_rhdl_compile --exact --nocapture --ignored 2>&1" -->


For booleans, the literals of `true` and `false` are used as usual:

```rust
{{#rustdoc_include ../code/src/kernels/literals.rs:step_4}}
```

```admonish note
At one point, RHDL supported custom suffixes to indicate the width of literals, e.g. `42_u4` meant you had a 4-bit integer.  This broke a couple of things

- Removing the `#[kernel]` annotation meant you no longer had valid Rust code.
- There are weird corner cases in which custom suffixes do not actually work.

Maybe in the future I'll bring them back?  But the function call notation isn't overly cumbersome.
```



