# State

Each `Circuit` element is assumed to have some internal state needed to simulate its response to some provided input.  You can use any type you want, provided it is cloneable and comparable.  The derive macro uses:

```rust
type S = (Self::Q, <child_0>::S, <child_1>::S,...)
```

The child states are probably expected.  The need to store `Self::Q` has to do with the fact that we need a value of `Q` to bootstrap the kernel in the simulation.  To explain why `S.0 : Q` be default, consider the canonical diagram:

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

Let us assume the circuit is in some state `S`, and we present a new input `I` at some time `t_0`.  The `Kernel` will update its outputs to reflect the new input, but to compute the output, it must also have a set of values for the child outputs.  These outputs (collectively known as the type `Q`) define part of the state of the circuit.  We must "remember" the value of `q: Q` between changes of the input signal, to effectively compute the output.  
