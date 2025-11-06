# Initialization

Initialization of `Synchronous` designs is completely analogous to `Circuit`.  For simulation, an initial state must be provided.  Depending on the behavior you want to model, this could consist of `dont care`, random values or fixed values.  The derive macro uses the following default:

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

Unlike the `Circuit` case, where the handling of the `reset` was left to you to do, in the case of `Synchronous`, resetting the circuit to a known state is baked into the design.  Every component receives both a `clock` _and_ a `reset` line.  The `reset` line can be used to provide a new state for the circuit.  

```admonish warning
Some (all?) FPGAs have a global reset that is automatically asserted when the device is programmed.  But be very careful when relying on that reset.  Some parts, for example, will reset all memory elements to zero on reset, which means that the initial condition you provide is silently ignored.  Other FPGA will take on the initial condition you provide on the reset.  The conservative, cross-platform solution is to manually implement the reset and hope that when possible the tools will optimize out the extra logic.

It is basically hardware Undefined Behavior to not reset your circuit to a known state.  Furthermore, it's hard to demonstrate that UB in simulation, where things tend to be deterministic.  If you really want to poke the bear, set up some kind of randomized initial state for your design and see if it always behaves correctly.  Or just implement a reset.
```
