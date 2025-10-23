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

```rust,kernel:literals
#[kernel]
fn kernel(a: b8) -> b8 {
    let c1 = b8(0xbe); // hexadecimal constant
    let c2 = b8(0b1101_0110); // binary constant
    let c3 = b8(0o03_42); // octal constant
    let c4 = b8(135); // decimal constant
    a + c1 - c2 + c3 + c4
}
```

In this example, we use a bare literal, and RHDL determines that it must be an 8-bit literal, as it is being added to an 8-bit literal:

```rust,kernel:literals
#[kernel]
fn kernel(a: b8) -> b8 {
    a + 42 // 👈 inferred as a 42 bit constant
}
```

If RHDL infers a size for your literal that won't hold the value you specify, you will get an error during the RHDL compilation of your kernel function, or a run time panic if you try to exercise the `kernel` manually.

```rust,kernel:literals
#[kernel]
fn kernel(a: b8) -> b8 {
    a + 270 // 👈 panics at runtime or fails at RHDL compile time
}
```

For booleans, the literals of `true` and `false` are used as usual:

```rust,kernel:literals
#[kernel]
fn kernel(a: bool) -> bool {
    (a ^ true) || false
}
```

```admonish note
At one point, RHDL supported custom suffixes to indicate the width of literals, e.g. `42_u4` meant you had a 4-bit integer.  This broke a couple of things

- Removing the `#[kernel]` annotation meant you no longer had valid Rust code.
- There are weird corner cases in which custom suffixes do not actually work.

Maybe in the future I'll bring them back?  But the function call notation isn't overly cumbersome.
```



