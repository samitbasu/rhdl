# Circuit Trait

Every circuit in RHDL is classified as either

- a `Circuit`, which is considered asynchronous (no single set of clock and reset lines)
- a `Synchronous` circuit, which must have a single clock and reset line for the components of that circuit

You can think of this as a segmentation between "blocking/async", much as we have in regular Rust.  While `Synchronous` circuits are far more common in practice, we start with `Circuit` because it imposes fewer constraints on the design, and is slightly easier to describe.  

The `Circuit` trait, along with the `CircuitIO` and `CircuitDQ` traits relate to the canonical diagram in the following manner:

```badascii
                CircuitIO::I    CircuitIO::Kernel          CircuitIO::O          
                      +                 +                          +         
                      |                 |                          |    
       +-------------+|+---------------+|+------------------------+|+---+        
       |              |                 v                          |    |        
 input |              v    +-----------------------+               v    | output 
+----->+------------------>|input            output+--------------------+------->
       | CircuitDQ::Q      |         Kernel        |  CircuitDQ::D      |        
       |     +        +--->|q                     d+-----+    +         |        
       |     |        |    +-----------------------+     |    |         |        
       |     +------->|                                  |<---+         |        
       |              |    +-----------------------+     |              |        
       | q.child_1 +> +----+o        child_1      i|<----+ <+ d.child_1 |        
       |              |    +-----------------------+     |              |        
       |              |                                  |              |        
       |              |    +-----------------------+     |              |        
       | q.child_2 +> +----+o        child_2      i|<----+ <+ d.child_2 |        
       |                   +-----------------------+                    |        
       |                                                                |        
       +----------------------------------------------------------------+        
                           ^                                                     
            impl Circuit +-+                                                     
```

We will start with the `Circuit` trait itself, and the cover the other traits next.  The `Circuit` trait requires the following information of a design:

- A type for the inputs and outputs to the circuit
- A type for the internal feedback inputs and outputs for components
- A `kernel` function to relate the inputs, outputs and feedback signals

The remainder of the `Circuit` trait is connected to the support mechanisms for synthesis and simulation.  Here is a break down of the `Circuit` trait:

```rust
{{#rustdoc_include ../code/src/circuits/traits.rs:circuit_trait}}
```

Even though you will rarely `impl Circuit` manually, it's important to understand how it works, and what happens under the hood when you tag your struct with `#[derive(Circuit)]`.  

