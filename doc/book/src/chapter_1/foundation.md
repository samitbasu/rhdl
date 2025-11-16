# The Foundation

Let's start with the most important part of understanding how RHDL represents
hardware designs.  RHDL uses a hierarchical, encapsulated design strategy in which
circuits are composed of sub-circuits, "glued" together with pure functions.  This
diagram is extremely important to understanding how RHDL works:

```badascii
       +----------------------------------------------------------------+        
       |                                                                |        
 input |                   +-----------------------+                    | output 
+----->+------------------>|input            output+--------------------+------->
       |                   |         Kernel        |                    |        
       |              +--->|q                     d+-----+              |        
       |              |    +-----------------------+     |              |        
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

A hardware component has a set a set of inputs and a set of outputs.  Inputs
arrive into the component, are processed by the internals of the component
and generate outputs.  

The internals of the component consist of the following pieces:

1.  A pure `kernel` function
2.  Zero or more child components

That's it.  And yet, with this model it is possible to construct components
of arbitrary complexity.  

The `kernel` function has a signature that looks like:

```rust, ignore
fn kernel(inputs: I, child_outputs: Q) -> (O, D)
```

where the types are:

1. `I` is the type that describes the shape of all inputs into the component.
2. `Q` is the type that describes the shape of all outputs of internal components.
3. `D` is the type that describes the shape of all inputs to internal components.
4. `O` is the type that describes the shape of all outputs out of the component.


Here is the foundational diagram, with annotations for the various types:

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

Before diving into an example, there is a slight peculiarity here.  Components
are structs, and child components are composed into parent components with a simple
`struct` definition.  For the diagram above, we would have something like

```rust, ignore
pub struct MyCircuit {
     child_1: ChildCircuitType1,
     child_2: ChildCircuitType2
}
```

The idiomatic Rust way to form the `Q` and `D` types would be to use tuples, so that
effectively:

```rust, ignore
type Q = (ChildCircuitType1::O, ChildCircuitType2::O)
type D = (ChildCircuitType1::I, ChildCircuitType2::I)
```

and while this works well for trivial examples, it does not scale well to components containing several internal child components.  So we add a bit of additional flexibility and some macro magic to make things cleaner.  More on that in a bit.
