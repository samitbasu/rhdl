# Metastability

Before going further into the details of synchronous circuit mechanics, let's take a brief detour into the realm of `metastability`.  Once you have stateful components (like flip flops and memories) in your circuit, you will be forced to think about clock domains.  RHDL forces this through the type system.  But if you aren't familiar with the problem, it's worth spending a bit to describe the danger of meta stability in your designs.  

```admonish warning
Metastability is very similar to "memory safety" in Rust.  There are invariants ("all references are valid") that are required to be held for the program to avoid UB.  If you violate those invariants, then bad stuff happens.  Metastability is similar.  You pinky-promise that all inputs and outputs will only change on clock edges.  If you violate that promise, bad stuff will happen to your circuit and possibly anything attached to it.  Synchronous circuits are like safe Rust.  If everything is synchronous, you don't really have to worry about it.  RHDL will ensure that all I/Os are synchronous to the given clock.  You only need to deal with asynchrony at the boundary. The same is true with Rust - the unsafe stuff is usually hidden behind a safe API.  But it exists, nonetheless.
```

Consider a super simple synchronous circuit with a single 1-bit flip flop:
```badascii
    +-+DFF+-+   
+-->|d    q +-->
    |       |   
+-->|clk    |   
    +-------+ 
```
Recall that this flip flop will sample the input right before the rising edge of the clock, and then update the output to correspond to this value, which it will hold until the next rising edge:
```badascii
   Sample   v           v           v           v         
            :           :           :           :         
            +-----+     +-----+     +-----+     +-----+   
 clk  +-----+     +-----+     +-----+     +-----+         
            :           :           :           :         
      +-+D1+--+--+ D2 +---+---+ D3 +--+---+ D4 +--+---    
 input+---+---+-----------+-----------+-----------+---    
            :           :           :           :         
           >:ε:<        :           :           :         
              :         :           :           :         
      +-+XX+--+--+ D1 +---+--+ D2 +---+--+ D3 +---+-+D4 --
output+-------+-----------+-----------+-----------+------+
              :           :           :           :       
   Update     ^           ^           ^           ^       
```
Here we show the output changing at some time ε after the rising edge of the clock, but conventionally, the output is usually shown to change at the rising edge, since the time between the clock edge and the output changing is usually very _very_ short.

There are many good resources on metastability, but let's look at a simplified example.  We have a circuit element that requires the input to be stable for some amount of time δt before the clock edge (called the "setup time").  Now suppose that we have an input that transitions close to the clock edge, within that time δt
```badascii
          +----+    +----+    +----+  
clk   +---+    +----+    +----+    +-+
                  <+> δt              
                  +------------------+
 d   +------------+                   
                    ??????????+------+
 q   +-------------+??????????        
```

The `??` states represent the fact that we cannot know for sure what state the flip flop will be in during this time interval.  Maybe the input changing from low to high arrived in time and will be interpreted as a `1` input, or maybe it didn't and it will be interpreted as a `0` input.  Or maybe the FF will end up somewhere in between, in a "1/2" state. And worse, any outputs connected to this FF will also be in an indeterminate state.

This phenomenon is known as `metastability`, and there are plenty of resources on the web available to learn more.  The "fix" is to note that for a real flop, it will (quickly) decide one way or the other as to what it's going to take a `true` or `false` value.  In that case, we can actually draw the diagram
```badascii
           +----+    +----+    +----+     
clk   +----+    +----+    +----+    +----+
         <+> δt                           
         +------------------+             
 d1   +--+                  +------------+

 (resolves false) +  
                  |                     
           ??     v  +---------+          
 q1   +---+??+-------+         +---------+
 
 (resolves true)  + 
                  v      
           ??+-----------------+          
 q1   +---+??                  +---------+
```
In the upper `q1` trace, the ff decided that the input was `false`, and so went low for the cycle.  In the lower `q1` trace, the ff decided that the input was `true` and so went high for the cycle.  

This should probably terrify you.  It basically means that you get a random bit output because of the violation of the timing for the flip flop.  The `fix` is to sample the output of the ff with another flip flop, (`q2`), which will _at least_ resolve to one of the two states at every clock edge. The diagram now looks like this:
```badascii
           +----+    +----+    +----+     
clk   +----+    +----+    +----+    +----+
         <+> δt                           
         +------------------+             
 d1   +--+                  +------------+


 (resolves false) +  
                  |                     
           ??     v  +---------+          
 q1   +---+??+-------+         +---------+
 
 (resolves true)  + 
                  v      
           ??+-----------------+          
 q1   +---+??                  +---------+
                                          

    (arrives late)             +---------+
 q2   +------------------------+          
                                          
    (arrives early)  +-------------------+
 q2   +--------------+                    
```

 The important part here is _not_ that the signal made it out through the second flop eventually.  The important part is that there are no `??` sections in the output `q2`.  So even though the bit may be lost, at least the uncertainty stops propagating.  Problem solved, right?

I've seen hardware designers put longer chains in place in the belief that this somehow avoids the problem.  It does not.  The damage was done at the first FF.  The subsequent flip flops are only to reduce the probability of a `??` appearing at the output.  This is already exceptionally unlikely as the amount of time the FF spends in the `??` state is generally quite short with respect to the clock cycle period.    However, a belt-and-suspenders approach suggests an extra FF doesn't hurt.

The root problem, however, remains.  The bit was lost in transition. So what to do?  The simplest answer is **Ensure this doesn't happen**.  Make sure that the input to your synchronous circuit only changes on the clock edge (or slightly after it) so that there is no uncertainty as to the value of the flip flops in your design.

The good news is that your toolchain will already make sure your design meets the required timing, and will move things around and insert delays until it does (or it will lower the rate at which the design can be clocked to allow for changes to settle and propagate).  As long as you work with `Synchronous` circuits, their outputs are intended to stabilize before the next clock edge, and so metastability is not a concern.  I bring it up because this simplification is the motivation for synchronous design in the first place.

```admonish warning
Note that `rhdl` does not simulate metastability.  Essentially for DFFs in `rhdl`, the value of δt is `0`.  So be extra cautious when assuming that your designs are safe if you are crossing clock domains or bringing in asynchronous signals without synchronizers.
```

For more detail, Wikipedia has good stuff.  [This](http://cva.stanford.edu/people/davidbbs/classes/ee108a/winter0607%20labs/lect.9.Metastability-blackschaffer.ppt) presentation is also excellent.  
