# Signals

There are only two types that `impl Timed` in the RHDL type system:

- `Signal<X, D>` where `X: Digital` and `D: Domain`.  This type represents a set of wires carrying data of type `X`, and that changes when dictated by `D: Domain`.  
- The empty type `()`.  Types that carry no data can be freely moved between time domains.

All remaining `impl Timed` are constructed by aggregating types that are `impl Timed`.  The usual tuples, arrays, and structs (with the appropriate trait bounds) all can be used to `impl Timed`.  An enum cannot be `impl Timed`.  But a signal carrying an enum can.  This is due to the nature of an enum.  The bits must all be taken together, or the invariants holding the discriminant to the payload could be violated at any time.

