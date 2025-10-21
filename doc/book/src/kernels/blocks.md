# Blocks

As in rust, all blocks in RHDL have a value, represented by the last expression in the block if it lacks a semi-colon, or the empty `()` value if no value is provided.  Block values are used in idiomatic rust, and you should feel free to use them in your RHDL designs as well.  Blocks allow you to create new scopes for bindings that won't conflict with the enclosing scope, and play nicely with control flow constructs like `if` and `match`.   But here's a minimal example:

```rust
#[kernel]
fn kernel(a: b8, b: b8) -> b8 {
    let c = {
        let d = a;
        let e = a + d;
        e + 3 // 👈 block value computed from this expression
    };
    a + c
}
```
