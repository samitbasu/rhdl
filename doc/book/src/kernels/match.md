# Match expressions

Match expression in Rust are quite powerful, and unfortunately, much of that power doesn't translate well into hardware designs.  So the version of the `match` expression supported in RHDL kernels is significantly simpler than the version provided by `rustc`.  That being said, you can still do a lot with `match` expressions.

```admonish warning
Currently, match guards are not supported.  That might change in the future, but for now, you will need to rewrite your code to remove match guards.
```


Here are the types of match arms that _are_ supported.

- Literal patterns, e.g. `0`.
- Pathed values, like `Bar::Baz::Val`, provided the named value is in scope, and `impl Digital`.
- In scope values which are constant, e.g. `ONE` or `TWO`.
- Wildcards
- Struct variant patterns, e.g., `Bar::Foo{a, b}`, where `Bar` is an enum with a `struct` variant named `Foo`.
- Tuple variant patterns, e.g., `Bar::Foo(a,b)`,  where `Bar` is an enum with a tuple variant named `Foo`.

All other types of match patterns are not supported.

The simplest form of a `match` is to build a lookup table.  In this case, the `match` patterns must be explicit in constructing patterns that match the type of the "scrutinee".  So this looks something like:

```rust
{{#rustdoc_include ../code/src/kernels/match_ex.rs:step_1}}
```

The syntax is a bit verbose here, but unfortunately, `rustc` does not allow the pattern match target to be a type alias (like `b8(0)`).  You have a couple of options to make this easier on the eyes.  The simplest is to extract the raw value of the `b8`, and match on that.  

```rust
{{#rustdoc_include ../code/src/kernels/match_ex.rs:step_2}}
```

This syntax works only because the RHDL compiler tracks the bit widths of the scrutinee and then coerces the various literal patterns into the proper widths.   A less dirty solution is to have names for the values.  Which might be more readable anyway:

```rust
{{#rustdoc_include ../code/src/kernels/match_ex.rs:step_3}}
```

In this case, an `enum` would definitely be a better choice.  But if you need to match against a set of literal values, one of these techniques will probably work for you.  If you are matching against values that come from outside RHDL (for example, if you have a 2-bit error signal that is read on a bus), I suggest you do something like this:

```rust
{{#rustdoc_include ../code/src/kernels/match_ex.rs:step_4}}
```

The namespace makes the pattern match clearer and easier to read.  It also means that if you need to reference magic constants like `ENDPOINT_ERROR`, you need only define it once in your code.

The `match` pattern syntax is most useful with `enum`s.  For example:

```rust
{{#rustdoc_include ../code/src/kernels/match_ex.rs:step_5}}
```

Here, we see the various forms of tuple struct variant and struct variant matching, with fields extracted through rebinding.  

```admonish warning
RHDL cannot currently pattern match nested `enum`s.  So a pattern like `Foo::Bar(SimpleEnum::Point{x: _, y})` will not work.  
```

The following is an example of trying to do a nested pattern match, which results in a compilation error from the RHDL compiler:

```rust
{{#rustdoc_include ../code/src/kernels/match_ex.rs:step_5b}}
```

With this test function:

```rust
{{#rustdoc_include ../code/src/kernels/match_ex.rs:step_5b_test}}
```

<!-- cmdrun to-html "cd ../code && cargo test --lib -- kernels::match_ex::step_5b::test_nested_match_compile_error --exact --nocapture --ignored 2>&1" -->


When working with structs and tuples, partial pattern matches are not supported. You can destructure the entire struct or tuple, but you don't need a `match` for that.  Just use a `let`

```rust
{{#rustdoc_include ../code/src/kernels/match_ex.rs:step_6}}
```

As a special case, you can also use `if let` to express a pattern match with a single pattern and a wildcard.  This is especially handy for dealing with `Option`:

```rust
{{#rustdoc_include ../code/src/kernels/match_ex.rs:step_7}}
```

```admonish note
Use `match` primarily for enums, or for small lookup tables.  That is what is best supported in RHDL and the closest to hardware translation.  Avoid the more sophisticated forms of match, as these are unlikely to translate to hardware.
```
