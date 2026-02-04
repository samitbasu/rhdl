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

Now to divide by 100 million, we cannot do it in one shot.  Let's assume that our counters are 8 bits wide.  Each 8 bit counter will overflow exactly once every 256 input clock cycles.  So after the first counter, we have a signal that pulses  at a frequency of 100MHz / 256 = 390.625 kHz.  If we feed the overflow of that counter into a second 8 bit counter, and take its overflow signal, we get a signal that pulses at a frequency of 390.625 kHz / 256 ~= 1.526 kHz.  A third counter will overflow at a rate of ~5.96 Hz.  That is an inconvenient frequency to blink an LED at, but it's not crazy.  We can at least see it.  However, we don't want to connect the overflow signal of the third counter directly to the LED - the overflow signal is still one clock cycle long, which is 10 nanoseconds.  Blink and you'll miss it.  Instead, we want to toggle the state of the LED every time the third counter overflows.  We can do that with a final DFF before the LED.  

So let's look at the total design:

```badascii
       +-------+       +-------+       +-------+       +-------+     +------+      
   +-->|en     |ovf +->|en     |ovf +->|en     |ovf +->|en     |ovf  |      | LED  
       |       +----+  |       +----+  |       +----+  |       +---->|      +--->  
       | 8bit  |       | 8bit  |       | 8bit  |       | 8bit  |     |Toggle|      
cr +-->| count |    +->| count |    +->| count |    +->| count |  +->|      |  ~3Hz
   |   +-------+    |  +-------+    |  +-------+    |  +-------+  |  +------+      
   |   R: 100MHz    |  R: 390KHz    |   R: 1562Hz   |   R: 5.96Hz |                
   |                |               |               |             |                
   +----------------+---------------+---------------+-------------+                
```

Later we will make it blink at the exact rate we want, but for now, this is close enough.  A few observations:

- Each of the counters is technically "running" at 100MHz.  That is to say, they are all clocked on the same clock domain.  The effective rate is lower because they are only incrementing when their enable signal is high, which only happens when the previous counter overflows.
- It's not good practice to directly manipulate clock signals with general fabric (counters, and logic) in a FPGA.  Clocks are special in that they need to clean and free of glitches.  That's why we do not attempt to divide the clock itself.  There are special blocks you can use for clock division.  
- The final toggle flip-flop is a simple way to get a slower blinking rate.  Each time the overflow from the last counter goes high, the toggle flip-flop changes state, effectively halving the blinking frequency.

As far as a design exercise for us, we need to design a toggle component, and an 8 bit counter, and then assemble three counters and the toggler together to make the final blinky circuit.  We'll start with the counter.

