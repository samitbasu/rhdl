# Match expressions

Match expression in Rust are quite powerful, and unfortunately, much of that power doesn't translate well into hardware designs.  So the version of the `match` expression supported in RHDL kernels is significantly simpler than the version provided by `rustc`.  That being said, you can still do a lot with `match` expressions.

```admonish warning
Currently, match guards are not supported.  That might change in the future, but for now, you will need to rewrite your code to remove match guards.
```


Here are the types of match arms that _are_ supported.

- Literal patterns, e.g. `0`.
- Struct patterns, e.g., `Foo {a, b}` where `Foo` is a struct that `impl Digital`.
- Struct variant patterns, e.g., `Bar::Foo{a, b}`, where `Bar` is an enum with a `struct` variant named `Foo`.
- Tuple Struct patterns, e.g. `Foo(a, b)` where `Foo` is a struct that `impl Digital`.
- Tuple variant patterns, e.g., `Bar::Foo(a,b)`,  where `Bar` is an enum with a tuple variant named `Foo`.
- Wildcards
- Pathed values, like `Bar::Baz::Val`, provided the named value is in scope, and `impl Digital`.
- In scope values which are constant, e.g. `ONE` or `TWO`.

All other types of match patterns are not supported.

Let's look at these in turn.  First, for literal patterns, RHDL applies type coercion to the literal to make it easy to use `bits` and literals.  For example:

```rust
#[kernel]
pub fn kernel(x: b8) -> b3 {
    match x {
        0 => b3(0),
        1 => b3(1),
        2 => b3(1),
        3 => b3(2),
        _ => b3(4),
    }
}
```
