# State

State is handled in the same way for `Synchronous` and `Circuit` designs.  As we discussed in the [state](../../circuits/circuit_trait/state.md) section, you can provide any associated type for your circuit that you need to hold the state of the circuit.  It needs only be cloneable and comparable.  The derive macro uses the exact same defition of the state type `S` as it does for a circuit:

```rust
type S = (Self::Q, <child_0>::S, <child_1>::S,...)
```

The rationale for this choice is the same as for `Circuit`.  See [state](../../circuits/circuit_trait/state.md) for the details.

```admonish note
The terminology for the feedback paths of internal components was chosen based on the simple flip flop.  Normally, the input to the flip flop is labelled as `D`, and the output is labelled `Q`.  Thus, it is somewhat idiomatic to assume that `D` represents the input we want to drive the next state of the flip flop, and `Q` as the current output of the flip flop. 
```
