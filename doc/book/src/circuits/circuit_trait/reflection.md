# Reflection

In the previous sections, when talking about `Descriptors` we looked primarily at a single, simple `kernel`.  But what happens with composition?  Any non-trivial `Circuit` will contain both a compute `kernel` to describe the interconnect and data flow, as well as children circuits that `impl Circuit` on their own.  Recall from the canonical diagram that circuits are hierarchically composed:

```badascii
            I                                                  O                 
            +                                                  +                 
       +---+|+------------------------------------------------+|+-------+        
       |    |                                                  |        |        
 input |    v              +-----------------------+           v        | output 
+----->+------------------>|input            output+--------------------+------->
       |                   |         Kernel        |                    |        
       |              +--->|q                     d+-----+              |        
       |   Q+-------->|    +-----------------------+     |<-----+D      |        
       |              |                                  |              |        
       |              |    +-----------------------+     |              |        
       | q.child_1 +> +----+o        child_1      i|<----+ <+ d.child_1 |        
       |              |    +-----------------------+     |              |        
       |              |                                  |              |        
       |              |    +-----------------------+     |              |        
       | q.child_2 +> +----+o        child_2      i|<----+ <+ d.child_2 |        
       |                   +-----------------------+                    |        
       |                                                                |        
       +----------------------------------------------------------------+        
```

So the `Descriptor` for our top level circuit must include the information about the child circuits.  This is why the last method in the `Circuit` trait provides a means of iterating over the child subcircuits:

```rust
fn children(&self, parent_scope: &ScopedName) -> impl Iterator<Item = Result<Descriptor<AsyncKind>, RHDLError>>;
```

If implementing it yourself, you will probably do something like:

```rust
fn children (&self, parent_scope: &ScopedName) -> impl Iterator<Item = Result<Descriptor<AsyncKind>, RHDLError>> {
    [
        self.child_1.descriptor(parent_scope.with("child_1"))
        self.child_2.descriptor(parent_scope.with("child_2"))
        // etc.
    ].into_iter()
}
```

There are a couple of details here, worth noting.

- The need for name scoping is because we want different "instances" of a given child circuit regardless of if the type signature is the same.  Each identical copy of a child circuit will still need an explicit instantiation in the resulting design.
- Building the parent descriptor will necessarily require the descriptors of the children.  RHDL handles this for you with the generic `build_asynchronous_descriptor<C: Circuit>` function.   The implementation is fairly simple, so I encourage you to go read it if you want to better understand the internals.

```admonish note
It's possible that the reflection mechanism will also be used for some kind of visualization in the future.  I would like to generate circuit diagrams as documentation at some point.  These will be driven off the runtime information in the `Descriptor`.
```

