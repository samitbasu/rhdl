# Arrays

RHDL includes support for fixed sized arrays.  In general, an implementation provides `Digital` for `[T; N]` where `T: Digital`.  So for example, you can write kernels that take an array of items as inputs:

```rust,kernel:arrays
#[kernel]
fn kernel(x: [b4; 4]) -> b6 {
    let mut accum = b6(0);
    for i in 0..4 {
         accum += x[i].resize::<6>();
    }
    accum
}
```

You can also return arrays as outputs or use them anywhere `Digital` values are allowed.

There are two ways of indexing arrays.  The first is based on indexes that are determined at compile time.  In the previous kernel, for example, the index to the array is known at compile time.  The same is true if you index with a literal, like `array[0]`.  In this case, the relevant bits of the value are known at compile time, and no hardware is required to extract this field from the array.  

However, it is also possible to index an array at "run time" using a dynamic index that comes from some other value.  For example, consider the following:

```rust,kernel:arrays
#[kernel]
fn kernel(x: [b4; 8], ndx: b3) -> b4 {
    x[ndx]
}
```

This _will_ generate hardware of some kind, since in general, you need to dynamically shift some nibble from the variable `x` so that it appears in the right place to select for the output.  This might be a barrel shifter which has a logarithmic number of stages depending on the dimension of the array.  These can be large, so be cautious with arrays that need dynamic indexing.  

Note that a barrel shifter is also implied by a shift operator that is controlled by a dynamic index.  For example, if we tried to select a bit out of a `b8` at runtime via:

```rust,kernel:arrays
#[kernel]
fn kernel(x: b8, ndx: b3) -> bool { 
           // 👇 - implies a barrel shifter
    (x & b8(1) << ndx) != 0
}
```

In general, use arrays when you can, as manual bit manipulation is error prone and hard to follow.  For example, if you have a gear reduction mechanism that takes blocks of 4 items and outputs 1 at a time, consider having the input `[T; 4]` and the output `T`.  This is much more ergonomic than trying to treat `[T; 4]` as a bit vector, and then doing the index arithmetic manually to extract different values of `T`.

RHDL also supports the `repeat` syntax, in which an array is built up from a single value and a length.  For example, you can write the following:

```rust,kernel:arrays
#[kernel]
fn kernel() -> [b4; 4] {
    [b4(3); 4]
}
```

The length of the repeat must be evident from the context.  It can be a `const` generic parameter, but must be evident at the site where the `repeat` is used.  Of course, you cannot assign arrays to each other if their lengths are different.  Inference of the length of the repeat is not currently supported.


