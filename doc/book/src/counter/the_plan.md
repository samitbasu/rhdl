# The Plan

The counter itself is very simple.

```badascii
      +--------------------------------> Overflow
      |    1                                     
      |    +                                     
      |    v                                     
carry |  +-----+                                 
      +--+ add |                                 
         |  w  |                                 
 sum  +-+|carry|<---+------------+               
      |  +-----+    |            |               
      |             |            |               
 +---+|+------------+            |               
 |    |                          |               
 |    |     +                    |               
 |    |     |\        +-+DFF+-+  |               
 |    +---->|1+       |       |  |               
 |          | |++---->|D     Q+--+------> Output 
 +--------->|0+       |       |                  
            |/        |       |                  
            +^        |cr     |                  
   Enable    |        +-------+                  
  +----------+         ^                         
   ClockReset          |                         
  +--------------------+                         
```

It consists of:

- An N-bit wide Digital Flip-Flop (DFF) to hold the current state of the counter.
- An N+1-bit wide Adder (ADD) that adds 1 to the current state of the counter.  The carry signal is used to indicate when the counter rolls over (i.e., when it goes from its maximum value back to zero).
- A 2-to-1 Digital Multiplexer (MUX) that selects between the current state of the counter (to hold the value) or the output of the adder (to increment the counter) based on the Enable signal.

The digital flip flop is a stateful element in RHDL that acts as a memory element.  On each clock edge, it captures the value at its D input and presents that value at its Q output until the next clock edge.  The flip-flop also has a ClockReset input that resets the output to zero when asserted.

Here is a quick sketch of what a DFF does:

```badascii
    +--+X1+---+--+X2+---+--+X3+---+--+X4+---+       
D   +---------+---------+---------+---------+       
           :         :         :         :          
           +----+    +----+    +----+    +----+     
CR    +----+    +----+    +----+    +----+          
           :         :         :         :          
      +----+--+X1+---+--+X2+---+--+X3+---+--+X4+---+
Q     +----+---------+---------+---------+---------+
                                                    
           t0        t1        t2        t3         

```

In this diagram, when the clock signal changes state from `false` to `true` (the rising edge of the clock) at time t0, the input to the DFF (labeled `D`) is sampled and the output `Q` is updated to reflect that value.  The output `Q` remains constant until the next rising edge of the clock at time t1, when the input `D` is again sampled and the output `Q` is updated accordingly.  This process continues for each rising edge of the clock.

Some things to note about this diagram:

- When you first power on a DFF, it will likely have an undefined value.  Some platforms force all DFFs to zero.  Others allow you to specify their initial value.  
- This is why you _need_ the reset signal.  Sending a reset to the DFF forces it into a known state.
- This DFF only captures the value on the rising edge of the clock.  You can also have DFFs that capture on the falling edge or both edges.  

While a DFF is itself an asynchronous circuit (it contains internal basic gates with feedback), we do not have a component-level model for it in RHDL.  Instead we have a primitive `DFF` type that can hold any `Digital` type.  You will find lots of DFF instances in your synchronous circuit designs.

In the case of the counter, the DFF holds the current count value.  On the clock edge, it samples the output of the MUX, which either holds the current count value (if Enable is low) or the current output plus one (if Enable is high).  This way, the counter increments its value on each clock edge when enabled, and holds its value otherwise.

Also, the `Counter` component in RHDL is generic over the width of the counter, but for simplicity, we will focus on an 8-bit counter in this tutorial.  We can still do useful stuff with an 8-bit counter - we will use our counter to blink an LED, as `blinky` is the Hello World of FPGA systems. 

The second part of the plan is to then chain some counters together to act as a high value decimator of the clock signal.  Our FPGA board has a 100MHz clock, and we want to blink the LED at about 1Hz.  That means we need to divide the clock by 100 million.  We do this by cascading several counters together, each one decimating the clock by a factor of 2^8.

```admonish note
It may be tempting to just use a really wide counter (like a 32 bit counter) and then use that to divide the clock.  Unfortunately, for our FPGA target (the iCE40), wide counters will not run fast enough to cope with the 100MHz source clock.  By breaking the counter into smaller pieces, we can ensure that each piece runs fast enough, and the overall design meets timing.
```
