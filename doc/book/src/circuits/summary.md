# Circuits

Circuits are the primary top level concept for describing hardware designs in RHDL.  RHDL uses a hierarchical, encapsulated design strategy in which
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

In this section, we will cover this diagram in greater detail, including how the composition works, the various types and traits used to describe the hardware design, and the simulation and synthesis of the circuit.