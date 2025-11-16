# Bitwise Logical Operators

The bitwise logical operators provided by RHDL are the same as those provided by `rustc`: OR, AND, XOR, and NOT.  So you can freely do all of the following:

```rust
let a : b8 = 0b0000_1011.into();
let b : b8 = 0b1011_0011.into();
let c = a | b; // 0b1011_1011
let d = a & b; // 0b0000_0011
let e = a ^ b; // 0b1011_1000
let f = !a;    // 0b1111_0100
```

The bitwise logical operators are also provided for signed values, although they make a lot less sense there.  But if for some reason you need to bit-manipulate a signed value, you can use all the same logical operators.

There are 3 reduction operators that are handy to use in hardware designs, these are ANY, ALL and XOR.  They are methods on both `Bits` and `SignedBits`.  They function as:

```rust
let a: b8 = 0b0000_1011.into();
let c: bool = a.any(); // equivalent to a != 0
let d: bool = a.all(); // equivalent to (!a) == 0
let e: bool = a.xor(); // true if an odd number of bits are 1, otherwise false
```

They are all synthesizable.
