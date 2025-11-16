# Operations

Nearly all hardware designs boil down to fairly low level bit manipulation.  This includes bit shifting, logical operations, arithmetic operations, etc.  As such, it is important to cover the nature of the supported operations in RHDL, and these are nearly all expressed as methods on the core `Bits` and `SignedBits` types.  In this section, we will walk through the operators, and provide insight into how the various operators behave, and how those behaviors might be different from `rustc` and from other HDLs.
