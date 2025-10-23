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

```rust,kernel:match
#[kernel]
pub fn kernel(x: b8) -> b3 {
    match x {
        Bits::<8>(0) => b3(0),
        Bits::<8>(1) => b3(1),
        Bits::<8>(3) => b3(2),
        _ => b3(5),
    }  
}
```

The syntax is a bit verbose here, but unfortunately, `rustc` does not allow the pattern match target to be a type alias (like `b8(0)`).  You have a couple of options to make this easier on the eyes.  The simplest is to extract the raw value of the `b8`, and match on that.  

```rust,kernel:match
#[kernel]
pub fn kernel(x: b8) -> b3 {
    match x.raw() {
        0 => b3(0),
        1 => b3(1),
        3 => b3(2),
        _ => b3(5),
    }  
}
```

This syntax works only because the RHDL compiler tracks the bit widths of the scrutinee and then coerces the various literal patterns into the proper widths.   A less dirty solution is to have names for the values.  Which might be more readable anyway:

```rust,kernel:match
pub const NO_DATA: b8 = b8(0);
pub const SINGLE: b8 = b8(1);
pub const MULTIPLE: b8 = b8(3);

#[kernel]
pub fn kernel(x: b8) -> b3 {
    match x {
        NO_DATA => b3(0),
        SINGLE => b3(1),
        MULTIPLE => b3(2),
        _ => b3(5),
    }
}
```

In this case, an `enum` would definitely be a better choice.  But if you need to match against a set of literal values, one of these techniques will probably work for you.  If you are matching against values that come from outside RHDL (for example, if you have a 2-bit error signal that is read on a bus), I suggest you do something like this:

```rust,kernel:match
//       👇 namespace the raw constants in a module
pub mod error_codes {
    use super::*;
    pub const ALL_OK: b2 = b2(0);
    pub const ENDPOINT_ERROR: b2 = b2(1);
    pub const ADDRESS_ERROR: b2 = b2(2);
    pub const RESERVED_ERROR: b2 = b2(3);
}

#[derive(Copy, Clone, PartialEq, Digital, Default)]
pub enum BusError { // 👈 Create a RHDL enum for the variants
    Endpoint,
    Address,
    #[default]
    Reserved
}

#[kernel]
pub fn kernel(x: b2, data: b8) -> Result<b8, BusError> {
    match x {
        error_codes::ALL_OK => Ok(data),
        error_codes::ENDPOINT_ERROR => Err(BusError::Endpoint),
        error_codes::ADDRESS_ERROR => Err(BusError::Address),
        error_codes::RESERVED_ERROR => Err(BusError::Reserved),
        _ => Err(BusError::Reserved), // 👈 unreachable but rustc doesn't know this    
    }
}
```

The namespace makes the pattern match clearer and easier to read.  It also means that if you need to reference magic constants like `ENDPOINT_ERROR`, you need only define it once in your code.

The `match` pattern syntax is most useful with `enum`s.  For example:

```rust,kernel:match
#[derive(PartialEq, Debug, Digital, Default, Clone, Copy)]
pub enum SimpleEnum {
    #[default]
    Init,
    Run(b8),
    Point {
        x: b4,
        y: b8,
    },
    Boom,
}

const B6: b6 = bits(6);

#[kernel]
fn kernel(state: SimpleEnum) -> b8 {
    match state {
        SimpleEnum::Init => bits(1),
        SimpleEnum::Run(x) => x,
        SimpleEnum::Point { x: _, y } => y,
        SimpleEnum::Boom => bits(7),
    }
}
```

Here, we see the various forms of tuple struct variant and struct variant matching, with fields extracted through rebinding.  

```admonish warning
RHDL cannot currently pattern match nested `enum`s.  So a pattern like `Foo::Bar(SimpleEnum::Point{x: _, y})` will not work.  
```

When working with structs and tuples, partial pattern matches are not supported. You can destructure the entire struct or tuple, but you don't need a `match` for that.  Just use a `let`

```rust,kernel:match
#[derive(Copy, Clone, PartialEq, Digital)]
pub struct Point {
    x: b8,
    y: b8,
}

#[derive(Copy, Clone, PartialEq, Digital)]
pub struct Reflect(pub Point);

#[kernel]
pub fn kernel(x: Reflect) -> b8 {
    let Reflect(p) = x;
    let Point {x, y: _} = p;
    x
}
```

As a special case, you can also use `if let` to express a pattern match with a single pattern and a wildcard.  This is especially handy for dealing with `Option`:

```rust,kernel:match
#[kernel]
pub fn kernel(x: Option<b8>) -> Option<b8> {
    if let Some(v) = x {
        Some(v + 1)
    } else {
        None
    }
}
```


```admonish note
Use `match` primarily for enums, or for small lookup tables.  That is what is best supported in RHDL and the closest to hardware translation.  Avoid the more sophisticated forms of match, as these are unlikely to translate to hardware.
```
