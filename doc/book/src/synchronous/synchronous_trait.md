# Synchronous Trait

The `Synchronous` trait, along with `SynchronousIO` and `SynchronousDQ` traits relate to the canonical diagram of the Synchronous circuit in the following manner:

```badascii
                 SynchronousIO::Kernel +-+                                       
        +-------------------------------+|+-----------------------------+        
        |   +--+ SynchronousIO::I        v  SynchronousIO::O +--+       |        
  input |   v               +-----------------------+           v       | output 
 +----->+------------------>|input            output+-------------------+------->
        | +---------------->|c&r      Kernel        |                   |        
        | |            +--->|q                     d+-----+             |        
        | |            |    +-----------------------+     |             |        
        | |            |                                  |             |        
        | |            |    +-----------------------+     |             |        
        | |q.child_1+> +----+o        child_1      i|<----+ <+d.child_1 |        
        | +-----------+|+-->|c&r                    |     |             |        
        | |            |    +-----------------------+     |             |        
        | |     +----->|                                  |             |        
  clock | |     +      |    +-----------------------+     |             |        
& reset | |q.child_2+> +----+o        child_2      i|<----+ <+d.child_2 |        
 +------+-+---------------->|c&r                    |    ^              |        
  (c&r) |       +           +-----------------------+    |              |        
        |       ++ SynchronousDQ::Q    SynchronousDQ::D ++              |        
        +---------------------------------------------------------------+        
```

Just as we did for the `Circuit` trait, we will start with the `Synchronous` trait itself.  The `Synchronous` trait requires the following information for a design:

- A type for the inputs and outputs to the circuit
- A type for the internal feedback inputs and outputs for components internal to the circuid
- A `kernel` function to relate the inputs, outputs and feedback signals.

The remainder of the `Synchronous` trait is related to simulation and synthesis of the circuit.  We can look at the `Synchronous` trait in its entirety.  It's pretty simple:

```rust
pub trait Synchronous: 'static + Sized + SynchronousIO {
    type S: PartialEq + Clone;
    fn init(&self) -> Self::S;
    fn sim(&self, clock_reset: ClockReset, input: Self::I, state: &mut Self::S) -> Self::O;
    fn descriptor(&self, scoped_name: ScopedName) -> Result<Descriptor<SyncKind>, RHDLError>;
    fn children(
        &self,
        _parent_scope: &ScopedName,
    ) -> impl Iterator<Item = Result<Descriptor<SyncKind>, RHDLError>>;
}
```

You will rarely implement this trait manually, since the `derive` macros make it much easier, but just as we did with `Circuit`, we will walk through this trait and the methods so you understand how it works, and what each piece does.