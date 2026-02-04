# Blinky


So far, our tutorial examples have all centered on asynchronous designs, in which signal inputs and outputs can change at any time.  Ultimately, all designs need to contend with asynchrony when they interface with the real world, but the vast majority of stuff you will encounter in digital design will be synchronous.  Synchronous designs use a clock signal to coordinate when signals can change, when their value should be sampled, and when internal state should be updated.  

In RHDL, our base foundational diagram changes a bit for synchronous designs.  The prevalance of a clock and reset signal that need to feed each and every element of a synchronous design means that we need to fan those signals out to each component in the design.  Originally, I required the design to fan those signals out manually and to wire them to each component that needed them (this is how RustHDL worked).  It's messy though and highly repetitive.  So now, RHDL special cases a synchronous circuit in which all elements/components are synchronized to a single clock and reset signal.

```admonish note
RHDL definitely allows you to have multiple clocks in your design.  The Synchronous pattern is meant to make it easy to create designs that run in a single clock domain.  When you start getting components running on multiple clocks, you will need to work with the more general (and verbose) Asynchronous pattern.  I'll cover some multi-clock designs later on.
```

The foundational diagram for a Synchronous circuit looks like this:

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

Here the `ClockReset` signal (which is a special type) is fanned out automatically to each child component of the circuit.  This also implies that the children of a synchronous circuit must themselves be synchronous.  

We also have a a simpler interface to a synchronous circuit.  Recall that in our previous designs, even the simple XOR gate had to deal with `Timed` inputs and outputs, so there were a lot of `Signal<X, Domain>` types floating around.  This was required because you could easily have a circuit that has to deal with, e.g., two inputs that belong to two different timing domains, and needed a way to tell RHDL which was which.

By contrast, in a synchronous circuit, all inputs and outputs _are assumed_ to belong to the timing domain defined by the clock signal that feeds the circuit.  This means that we can use simple `Digital` types for inputs, outputs and internal variables, and ignore the `Signal<_, _>` abstraction.  That makes the circuit easier to describe and reason about.

In this module, we will build a simple rolling counter with an enable signal.  When the enable is high, the counter will increment, and when the enable is low, the counter will hold its value.  The counter will reset to zero when the reset signal is asserted.  It's a super simple synchronous circuit, but it still has almost all of the important elements we need to cover synchronous design in RHDL.
