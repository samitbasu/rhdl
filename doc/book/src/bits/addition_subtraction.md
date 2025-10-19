# Addition and Subtraction

Addition and subtraction in RHDL are based on standard 2's complement, wrapping operators.  There are good resources on the internet if you are unfamiliar with the oddities of 2's complement representations, but they are the simplest from a hardware implementation.  If you need something like BCD (binary coded decimal) arithmetic, you will need to implement it yourself. 

The easiest way to think about RHDL's arithmetic operators is that they are all `Wrapping`.  That is to say, if an operation between two values of length `N` will not fit in a bit vector of length `N`, the lower `N` bits are kept and the remaining bits are discarded.  The results can be surprising.  For example in unsigned bit vectors, exceeding the maximum representable value will cause the values to wrap, like so:

```rust
let a: b8 = 255.into();
let b: b8 = a + 1; // No error.... but 
assert_eq!(b, b8(0));  // b is zero!
```

Similarly, underflowing with subtraction of unsigned values will cause wrapping behavior as well

```rust
let a: b8 = 0.into();
let b: b8 = a - 1; // No error.... but
assert_eq!(b, b8(255)); // b is b8::MAX
```

With signed values there are additional caveats.  A signed value of `N` bits can represent values in the range `-2^(N-1)..(2^(N-1)-1)`. This range is asymmetric, which means that you can cause wrapping even with the negation operator.  For example:

```rust
let a: s8 = (-128).into(); // This is OK
let b: s8 = -a; // No error.... but
assert_eq!(b, s8(-128)); // The operator did nothing
```

```admonish note
To explain that weird behavior, note that in 2's complement, the negation of a value involves inversion of the bits, followed by adding one.  So if we take `-128`, which has the representation:
```
-128      --> 1000_0000
!(-128)   --> 0111_1111
!(-128)+1 --> 1000_0000
```
In this case, the problem is that adding `1` to the value `0111_1111` causes it to overflow from a positive number to a negative one.  
```

In any case, I just want you to be aware of this behavior, as it is core to how RHDL handles arithmetic, and is meant to model how real hardware actually works.  If you want to ensure that your operations cannot underflow or overflow, you either need to make sure you have enough bits to represent all possible values, or use more advanced techniques like the extended operators, covered later.

Note also that the `assign op` versions of the operators also work in RHDL, provided you have a mutable target for the operation.  So you can do:

```rust
let mut a: b8 = 42.into();
a += 1; // a is now 43
```

