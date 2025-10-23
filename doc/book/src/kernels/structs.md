# Structs

Structs are generally well supported in RHDL, and most of the things you expect to be able to do with structs in rust can be done in RHDL.  For example, you can create a struct using the usual struct syntax, and then access it's members by name or by number (if it is a tuple struct).

```rust
#[derive(Copy, Clone, PartialEq, Digital)]
pub struct MyStruct {
    a: b8,
    b: b4,
    c: bool
}

#[kernel]
pub fn kernel(a: b8) -> MyStruct {
    let mut t = MyStruct {
        a,
        b: b4(1),
        c: a == 0
    };
    t.b = b4(2);
    t
}
```

```rust
#[derive(Copy, Clone, PartialEq, Digital)]
pub struct MyOtherStruct(pub b8, pub b8, pub b8);

#[kernel]
pub fn max_val(t: MyOtherStruct) -> b8 {
    let mut ret = t.0;
    if t.1 > ret {
        ret = t.1;
    }
    if t.2 > ret {
        ret = t.2;
    }
    ret
}
```

You can also use destructuring to take `struct`s apart with a `let`:

```rust
#[derive(Copy, Clone, PartialEq, Digital)]
pub struct MyStruct {
    a: b8,
    b: b4,
    c: bool
}

#[kernel]
pub fn kernel(t: MyStruct) -> b4 {
    let MyStruct {a, b, c} = t;
    if c {
        b
    } else {
        a.resize::<4>()
    }
}
```

If you only need some of the fields of the struct, you can ignore the others with `..`

```rust
#[derive(Copy, Clone, PartialEq, Digital)]
pub struct MyStruct {
    a: b8,
    b: b4,
    c: bool
}

#[kernel]
pub fn kernel(t: MyStruct) -> b4 {
    let Mystruct {b, ..} = t;
    b
}
```

RHDL also supports "functional update syntax", where you can provide missing fields from one struct definition with another.  For example:

```rust
#[derive(Copy, Clone, PartialEq, Digital, Default)]
pub struct MyStruct {
    a: b8,
    b: b4,
    c: bool
}

#[kernel]
pub fn kernel(a: b8) -> MyStruct {
    MyStruct {
        a,
        .. MyStruct::default()
    }
}
```

You can destructure tuple structs in a completely analogous way as well:

```rust
#[derive(Copy, Clone, PartialEq, Digital, Default)]
pub struct Color(pub b6, pub b6, pub b6);

#[kernel]
pub fn red2(c: Color) -> b7 {
    let Color(red, ..) = c;
    red.resize::<7>() << 1
}
```

Structs can be packed into arrays, used in enums, etc.  They can be very handy for collecting values together and treating them as a single entity in your design.  For example, if you have a hardware design that processes a memory write, it may make sense to put the memory address and data together.  And this struct may be generic over the widths of the memory and the type of data being held.  So we might have:

```rust
#[derive(Copy, Clone, PartialEq, Digital)]
pub struct Request<T, const N: usize>
where
    T: Digital,
    rhdl_bits::W<N>: BitWidth,
{
    data: T,
    address: Bits<N>,
}
```

Provided the bounds are given, this struct will behave like any other `impl Digital` data structure, and can be stored in memories, queues, etc.