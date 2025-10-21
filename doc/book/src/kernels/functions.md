# Functions

Ultimately, any circuit `kernel` must be a function, but you can also create other functions, annotate them with `kernel` and then call from from other kernels.  This form of composition allows you to factor your code into reusable pieces that can be individually and rigorously tested and then easily composed.

In general functions used in `kernel` must satisfy the following:

1. They must be `pub`.  This requirement is due to how the generated types get exported.
2. They cannot return a zero-sized type.  I.e., a synthesizable function must produce _something_.  The argument list can be empty, but not the return type.
3. Each argument must `impl Digital`.  The return type must also `impl Digital`.
4. There is a limit to the number of arguments the function can take.  Currently about 6.

Let's work through some examples so you can get a sense of what kinds of function signatures are allowed, and which ones are not.

The first is a simple function with a couple of arguments, a couple of returns.  Nothing too exciting.

```rust
#[kernel]
pub fn kernel(a: b8, b: b8) -> (b8, bool) {
    (a, b == a)
}
```

The following is _not_ allowed, because it returns a zero-sized type

```rust
#[kernel]
pub fn kernel(a: b8, b: b8) -> () {
    ()
}
```

You can use infallible pattern matching in the arguments for tuples.  Struct and enum patterns are currently unsupported.

```rust
#[kernel]
pub fn kernel((a,b): (b8, b8)) -> b8 {
    a + b
}
```

You can make your function generic and add bounds as needed.

```rust
#[kernel]
pub fn kernel<const N: usize>(a: Bits<N>, b: Bits<N>) -> Bits<N> 
    where W<N> : BitWidth {
    a + b
}
```

```admonish note
When calling a generic function from within a kernel, due to limitations of how the `#[kernel]` attribute works, you will generally need to supply all of the generic types.  Type inference across function calls is not great in RHDL at the moment.  This can be annoying, but it may improve in the future as the RHDL compiler improves.
```

Of course, you can pass `Digital` values into functions, and return `Digital` values out of the functions:

```rust
#[derive(Copy, Clone, PartialEq, Digital)]
pub MyStruct {
    x: b8,
    y: b8
}

#[kernel]
pub fn kernel(s: MyStruct) -> MyStruct {
    MyStruct {
        x: s.x + s.y,
        y: s.y
    }
}
```

```admonish warning
You cannot use `impl Digital` in return position with `kernel` functions.  The result can be generic, but it must be declared as a generic parameter.
```

You can _call_ functions just like you normally would in rust, provided the named function is available at the call site, and that the referenced function is annotated with `#[kernel]`.  So, for example:

```rust
#[kernel]
pub fn my_add(a: b8, b: b8) -> b8 {
    a + b
}

#[kernel]
fn kernel(a: b8, b: b8, c: b8) -> b8 {
    let p1 = my_add(a,b);
    let p2 = my_add(p1, c);
    p2
}
```

When the function you are calling is generic, you must provide the generic parameters explicitly at the call site.  This is due to a limitation on how the RHDL compiler works.  So if we had instead:

```rust
#[kernel]
pub fn my_add<const N: usize>(a: Bits::<N>, b: Bits:<N>) -> Bits::<N> 
where W<N> : BitWidth {
    a + b
}

#[kernel]
fn kernel(a: b8, b: b8, c: b8) -> b8 {
    //               👇 Must be explicit here!
    let p1 = my_add::<8>(a, b);
    let p2 = my_add::<8>(p1, c);
    p2
}
```

Getting type inference to work at these sites would involve some pretty substantial changes to the way the RHDL compiler works.  It's not impossible, so maybe in a future version.