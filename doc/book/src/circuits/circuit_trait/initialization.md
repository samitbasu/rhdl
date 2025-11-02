# Initialization

For simulation, the circuit must be able to provide an initial state.  Depending on the behavior you want to model, this could consist of `dont care` values, random values, or fixed values.  The derive macro uses:

```rust
fn init(&self) -> S {
    (
        <Self::Q as Digital>::dont_care(),
        self.child_0.init(),
        self.child_1.init(),
        ...
    )
}
```

The use of a `dont_care` value here raises a good point.  In some ways, the outputs of the child circuits are a `MaybeInit` value on power up.  This isn't necessarily reflected in RHDL, and could lead to a divergence between what you simulate and what you see in practice.  Normally, the way you handle this is with a proper `reset` signal.  If you don't have one then either:

- You are working with an FPGA that already provides a global reset when it is programmed, so that doesn't matter.
- You are designing something for which all possible inputs must be valid, and thus cannot assume any power-on-state (consider a combinatorial circuit for example), and all internal state is transient.
- You like to live dangerously.

Whatever the case may be, I decided early on not to try to model tri-valued logic everywhere, since there is no real Rust equivalent (short of making everything `MaybeInit` and hence `unsafe`).  If you want to fuzz test your circuit and do not have a reset, then consider implementing the `init` method yourself, and providing a random value as the initial state.  Even that won't be completely exhaustive, as Rust guarantees that enums are valid representations at all times, and at power up, it's possible this will not be true within a circuit.

```admonish warning
It is basically hardware Undefined Behavior to construct circuitry that doesn't have a proper reset.  The presence of the reset is what ensures that the circuit starts operation in a well defined state.  The simulation will not "fix" this for you.  The only real way to fix it is to include a reset signal.
```