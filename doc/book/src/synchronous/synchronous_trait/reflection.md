# Reflection

Reflection is another feature of the `Synchronous` trait that operates exactly like the `Circuit` case as described [here](../../circuits/circuit_trait/reflection.md).  In summary, we deal with composition in the circuit diagram below:

```badascii
        +---------------------------------------------------------------+        
        |   +--+ SynchronousIO::I           SynchronousIO::O +--+       |        
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
        | |            |                                  |             |        
  clock | |            |    +-----------------------+     |             |        
& reset | |q.child_2+> +----+o        child_2      i|<----+ <+d.child_2 |        
 +------+-+---------------->|c&r                    |                   |        
  (c&r) |                   +-----------------------+                   |        
        +---------------------------------------------------------------+        
```

At run time, we can retrieve `Descriptor` structs for the child circuits using the `children` method, which provides an iterator over the child circuits, returning a descriptor for each one:

```rust
    fn children(&self, parent_scope: &ScopedName) -> impl Iterator<Item = Result<Descriptor<SyncKind>, RHDLError>>;
```

For the diagrammed circuit above, this method would probably look something like:

```rust
fn children(&self, parent_scope: &ScopedName) -> impl Iterator<Item = Result<Descriptor<SyncKind>, RHDLError>> {
    [
        self.child_1.descriptor(parent_scope.with("child_1"))
        self.child_2.descriptor(parent_scope.with("child_2"))
        // etc.
    ].into_iter()
}
```

As in the case for `Circuit`, we need name scoping here to allow for encapsulation of the child subcircuits within the scope defined by the containing circuit.  Because the HDL modules live in a flat namespace, we rename them with a path to avoid collisions.  Building a parent descriptor needs the child descriptors.  This is handled by RHDL using the generic `build_synchronous_descriptor<S: Synchronous>` function.  

```admonish note
The reflection mechanism could be used for some kind of visualization mechanism or to generate visualizations of the circuit internals.  
```