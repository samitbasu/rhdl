# Tuples

Tuples are well supported in RHDL, and there are generic impls that mean any tuple of `Digital` elements is itself `Digital` (up to some length).  Tuples can be formed and deconstructed at will within your code.  There is not much more to say.  Here is a simple example:

```rust,kernel:tuples
#[kernel]
pub fn kernel(a: (b8, b8)) -> b8 {
    let (x, y) = a;
    let z = (x, x, y);
    let a = z.0 + z.1 + z.2;
    a + 1
}
```

