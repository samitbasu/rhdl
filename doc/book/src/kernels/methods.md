# Method calls

In general, method calls of the form `x.bar()` are relatively difficult to support in RHDL due the fact that type inference in RHDL happens at run time, which is too late to request the code for the relevant method.  Thus, apart from a small number of hand-coded methods that are special cased in the RHDL compiler, you cannot use methods.  If at some point in the future, the scope of the compiler expands to cover more of the work to process the source code (currently handled by `rustc`), this restriction might be lifted.  But for now, prefer the functional style of writing functions that manipulate pieces of data by value.  It tends to make testing easier anyway.

For the handful of method calls that are currently supported in RHDL, here is the list.  In the table below:

- `B` is short for `Bits::<N>`
- `S` is short for `SignedBits::<N>`
- `D` is short for `DynBits`
- `Y` is short for `SignedDynBits`

|Method|Example|Result|Domain|
|---|---|---|---|
|`any`|`x.any()`|`true` if any of the bits in `x` are nonzero|`BSDY`|
|`all`|`x.all()`|`true` if all of the bits in `x` are nonzero|`BSDY`|
|`xor`|`x.xor()`|`true` if an odd number of bits in `x` are nonzero|`BSDY`|
|`as_signed`|`x.as_signed()`|Reinterpret the bits as a signed value of the same bit-length|`BD`|
|`as_unsigned`|`x.as_unsigned()`|Reinterpret the bits as an unsigned value of the same bit-length|`SY`|
|`val`|`x.val()`|Extract the underlying value `T`|`Signal<T, D>`|
|`resize`|`x.resize::<N>()`|Resize a bit vector to the given length.  Sign aware for signed types.|`BSDY`|
|`raw`|`x.raw()`|Extract the raw underlying 128-bit value.|`BSDY`|
|`xadd`|`x.xadd(y)` |Bit preserving addition|`BSDY`|
|`xsub`|`x.xsub(y)`|Bit preserving subtraction| `BSDY`|
|`xneg`|`x.xneg()` |Bit preserving negation| `SY`|
|`xmul`|`x.xmul(y)` |Bit preserving multiplication| `BSDY`|
|`xext`|`x.xext::<N>()`|Bit preserving length extension|`BSDY`|
|`xshl`|`x.xshl::<N>()`|Bit preserving left shift|`BSDY`|
|`xshr`|`x.xshr::<N>()`|Bit discarding right shift|`BSDY`|
|`xsgn`|`x.xsgn()`|Bit preserving sign conversion|`BD`|
|`dyn_bits`|`x.dyn_bits()`|Convert to a runtime sized bitvector|`BS`|
|`as_bits`|`x.as_bits::<N>()`|Convert to a compile time sized bitvector|`D`|
|`as_signed_bits`|`x.as_signed_bits::<N>()`|Convert to a compile time sized signed bitvector|`Y`|

These method calls are all special cased inside the RHDL compiler.  There are checks to ensure that you don't try to `impl` these on your own types, but mostly these make no sense for aggregate data structures.  

```rust
{{#rustdoc_include ../code/src/kernels/methods.rs:step_1}}
```

