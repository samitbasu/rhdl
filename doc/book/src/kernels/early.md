# Early returns and Try expressions

RHDL supports both early returns and `try` expressions.  To make this work with late type inference is a bit tricky, and you may find that in some instances, you need to add type signatures to help RHDL understand the types involved, even though `rustc` is able to figure them out.  First off, you can early return from a kernel function, as you would expect:

```rust,kernel:early
#[kernel]
fn kernel(a: b8, b: bool) -> b8 {
    let c = a + 1;
    if b {
        return c; // 👈 Early return
    }
    let c = c + a;
    c
}
```

RHDL will add the needed control flow and placeholders to shortcut the remainder of the kernel when an early return is encountered.  The second way to early return is with a `try` expression.  For the most part, these also function the way you would expect.  You can have a `kernel` that returns a `Result<O, E>`, provided `O: Digital` _and_ `E: Digital`.  You can also have a kernel that returns a `Option<T>` where `T: Digital`.  In both cases, if you apply `?` to an expression, if can early return, just as in normal Rust.  This feature is quite handy for adding non-trivial error handling to your hardware designs.  Coupled with `enum`, you can make your code far clearer and error-explicit.  Here is an example with `Result`:

```rust,kernel:early
#[derive(Copy, Clone, PartialEq, Digital, Default)]
pub enum InputError {
    TooBig,
    TooSmall,
    #[default]
    UnknownError
}

#[kernel]
pub fn validate_input(a: b8) -> Result<b8, InputError> {
    if a < 10 {
        Err(InputError::TooSmall)
    } else if a > 200 {
        Err(InputError::TooBig)
    } else {
        Ok(a)
    }
}

#[kernel]
pub fn kernel(a: b8, b: b8) -> Result<b8, InputError> {
    let a = validate_input(a)?;
    let b = validate_input(b)?;
    Ok(a + b)
}
```

Nice, right?  Here is an example with Option, which might be better for instances in which failures should be silently discarded.

```rust,kernel:early
#[kernel]
pub fn kernel(a: Option<b8>) -> Option<b8> {
    let a = a?;
    Some(a+1)
}
```

```admonish note
Using `Option` in hardware designs is a good pattern. Normally, one has some kind of data lines, and then a `valid` strobe that indicates that there is valid data on the lines.  It is your problem to ensure that you read the data lines only when the `valid` strobe is asserted.  You can change this into an `Option<T>`, in which case RHDL will ensure that you can only see the data when the contents are `Some`.  Thus guaranteeing that the invariant promised by the protocol (only look at data when valid is `true`) is upheld by the compiler in accordance with the type system.
```
