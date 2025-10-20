# Variables

RHDL supports the definition of local variables within kernels.  The most straightforward is to use a `let` binding:

```rust
#[kernel]
pub fn kernel(a: b8, b: b8) -> bool {
    let c = a == b; // type is inferred as bool
    let d = !c;
    d
}
```

In this case the `let` binding creates a locally scoped binding `c`, which allows you to refer to the result of the comparison for it's use in the not operation that defines `d`.  

You can make bindings mutable, so that you can reuse them.  For example:

```rust
#[kernel]
pub fn kernel(a: b8, b: b8) -> bool {
    let mut c = a + 1;
    c += a; // mutates c
    c == b
}
```

In case you find this surprising (coming from other HDLs), you can simply think of this as a "trick" that creates a new binding at the assignment location that happens to also be called `c`.  In practice, this is what actually happens.  RHDL then does the bookkeeping to determine which `c` is being referred to at any point in the function.  Something like the following:

```badascii
       +-----+                
a+---->|     | c(1)+-----+    
       | Add +---->|     |    
1+---->|     |     |     |    
       |     |     | Add |c(2)
       +-----+     |     +--->
                   |     |    
         a+------->|     |    
                   +-----+    
```

where `c(1)` and `c(2)` refer to the two different definitions of `c` in the program.  

You can also use `let` to define a variable with no value, as long as you unconditionaly assign it a value before the termination of the function.

```rust
#[kernel]
pub fn kernel(a: b8, b: b8) -> bool {
    let c;
    let d = a + b;
    c = d;
    c == a
}
```

This form is usually handy when working with `if` expressions where you want to make assignments within the different branches of the `if`.

```admonish note
Names must be valid Rust identifiers.
```

You can also give an explicit type to the binding.

```rust
#[kernel]
pub fn kernel(a: b8, b: b8) -> b8 {
    let c: b8 = a;
    c
}
```

You can also use irrefutable destructuring with `struct`s and `tuples` (but not enums) when assigning bindings.  Here is an example with a normal struct definition:

```rust
#[derive(PartialEq, Clone, Copy, Digital)]
struct Foo {
    a: b8,
    b: b8,
}

#[kernel]
pub fn kernel(arg: Foo) -> b8 {
    let Foo {a, b} = arg;
    a + b
}
```

And here is one with a tuple struct:

```rust
#[derive(PartialEq, Clone, Copy, Digital)]
struct Foo(b8);

#[kernel]
pub fn kernel(arg: Foo) -> Foo {
    let Foo(a) = arg;
    Foo(a + 1)
}
```

You can also destructure normal tuples, provided each element `impl Digital`, of course:

```rust
#[kernel]
pub fn kernel(a: (b8, b8)) -> bool {
    let (c, d) = a;
    c == d
}
```


