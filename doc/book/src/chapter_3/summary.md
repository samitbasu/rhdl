# Chapter 3 - A Half Adder

Our next step will be to construct a half adder.  While not particularly exciting in and of itself, it will demonstrate the vital importance of circuit _composition_.  In RHDL, we build circuits up from smaller components.  And that hierarchical implementation imposes some structure on our design.  Due to some issues in how the code generation works, there are also some silent conventions that need to be followed.  We can explain all of that as we go.

Recall that a half adder implements addition for 2 1-bit signal, and computes the sum and carry out.  Internally, a half adder looks like this:
```badascii
              +---------+         
a +---+------>|         |         
      |       | XorGate +--> sum  
      |  +--->|         |         
      |  |    +---------+         
      |  |                        
      |  |                        
      |  |    +---------+         
      +-+|+-->|         |         
         |    | AndGate +--> carry
b +------+--->|         |         
              +---------+         
```

Referring back to our foundational diagram, we can now modify it with concrete components:

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
       | q.child_1 +> +----+o        XorGate      i|<----+ <+ d.child_1 |        
       |              |    +-----------------------+     |              |        
       |              |                                  |              |        
       |              |    +-----------------------+     |              |        
       | q.child_2 +> +----+o        AndGate      i|<----+ <+ d.child_2 |        
       |                   +-----------------------+                    |        
       |                                                                |        
       +----------------------------------------------------------------+        
```

Before constructing the top level circuit, we take a quick detour to make the `AndGate`.
