# Descriptors

Just like `Circuit`, the most important method in the `Synchronous` trait is the one that computes a `Descriptor` for the circuit:

```rust
fn descriptor(&self, scoped_name: ScopedName) -> Result<Descriptor<SyncKind>, RHDLError>;
```

The description of the `Descriptor` type and what the fields mean is identical to the one presented [here](../../circuits/circuit_trait/descriptors.md).  The only difference is the marker type used to indicate that the returned `Descriptor` is for a `Synchronous` circuit.  This marker type makes it hard to use a descriptor for a `Synchronous` circuit when an asynchronous descriptor is expected and visa versa.
