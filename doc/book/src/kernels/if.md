# If Expressions

RHDL supports the usual syntax for `if` expressions from Rust.  You can use them in the `C` style as statements, or as expressions.  Both are supported.  Here is an `if` as a statement:

```rust
#[kernel]
fn kernel(a: b8, b: b8) -> b8 {
    let c;
    if a > b {
        c = 3;
    } else if a == b {
        c = 5;
    } else {
        c = 7;
    }
    c
}
```

You can simplify this into an expression, and eliminate the binding for `c`, since the function block will take the value of the last expression:

```rust
#[kernel]
fn kernel(a: b8, b: b8) -> b8 {
    if a > b {
        3
    } else if a == b {
        5
    } else {
        7
    }
}
```

```admonish note
Just as rust has no ternary operator `?`, RHDL doesn't have one either.  Use an `if` expression, as it is generally easier to read, and less likely to cause confusion.
```

RHDL also supports `if let` (although not chaining).  So you can also use the following:

```rust
#[kernel]
pub fn kernel(data: Option<b8>) -> Option<b8> {
    if let Some(data) = data {
        Some(data + 1)
    } else {
        None
    }
}
```

Note that RHDL includes some special support for `Option` and `Result`, so there is a bit of extra type inference magic here to make this work.  But it also works with any other `enum` you want to pattern match on:

```rust
#[derive(Copy, Clone, PartialEq, Digital, Default)]
pub enum MyEnum {
    Red(b8),
    Green(b8, b8, b8),
    #[default]
    Blue,
}

#[kernel]
pub fn kernel(data: MyEnum) -> b8 {
    if let Red(x) = data {
        x
    } else {
        b8(42)
    }
}
```

