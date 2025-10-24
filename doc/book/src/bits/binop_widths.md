# Bit Widths and Binary Operators

One important thing to note for the core operators provided for `Bits` and `SignedBits` is that they fall into one of two types:

1.  Bit width preserving operators.   These operators take two elements of bit-width `N`, and return a new element of bit-width `N`.  For unary operators, there will a single input of bit-width `N` for the corresponding output of bit-width `N`.
2.  Reduction operators.  These operators take a quantity of `N` bits, and return a `bool`.  This category includes comparison operators, but it also includes methods like `.any()` and `.all()`.

Bit width preservation is pretty natural for "regular" Rust.  But there are some subtleties.

1.  There is no such thing as a panic in a hardware design (unless you build one yourself).  So while `rustc` may cause your program to panic if it creates an out-of-range result, the RHDL operators will simply wrap the result (i.e., through away bits).
2.  Bit widths can be tricky in some HDLs, as they attempt to preserve all information, or infer the number of bits needed, or pad arguments as needed to make operations make sense.  If you are used to `Rust`'s strict type rules, you will find RHDL to make sense.  You cannot add a `b4` to a `b8`, anymore than you can add a `u8` to a `u32`.

There _are_ times you want "all the bits".  For example, if you want the carry out bit when adding two bit vectors, how do you do this?  An ALU may need to calculate the carry out on the sum of two 32-bit values.   There are two ways to do this in RHDL.  The first (and maybe easiest) is to simply extend the to operands by an extra bit, and then perform the operation.  The carry bit is then stashed in the MSB of the resulting 33-bit number.  

```rust
let a = b8(200); 
let b = b8(100);
let a = a.resize::<9>(); // zero extend to 9 bits
let b = b.resize::<9>(); // ditto
let c = a + b;
let carry = get_msb::<9>(c);
```

This may look somewhat inefficient, but it will reduce down to a simple adder with the carry out bit residing in `carry`.  The rest (like adding zero, or anding with `0`) will be optimized out.  The `get_msb` function might look something like this:

```rust
#[kernel]
fn get_msb<const N: usize>(a: Bits<N>) -> bool
where
    rhdl::bits::W<N>: BitWidth,
{
    (a & (1 << (N - 1))).any()
}
```

For reduction operators, like `.any()`, `.all()` and `.xor()`, the output will be a single `bool` value.

That's basically it.  In general, all ops take one of the following forms:

1. `N op N --> N` (e.g., `+`)
2. `op N --> N` (e.g., `!`)
3. `N op N --> bool` (e.g., `>=`)
4. `N.op() --> bool` (e.g., `.any()`)

More complicated operations are possible using dynamic-width bit vectors, but that's an advanced topic and covered later.

