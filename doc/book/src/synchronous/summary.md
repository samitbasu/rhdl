# Synchronous Circuits

While the `Circuit` abstraction described in [Circuit](../circuits/summary.md) can, in principle describe arbitrary circuitry, it is in some sense _too_ general.  There are no assumptions about the how the inputs and outputs are related to each other, nor are there any enforced concepts of time or state change.  

Almost all hardware designs fall into a subset of circuit design that I call `Synchronous` for lack of a better term.  These circuits are characterized by the following:

- The circuit has a single clock input that is used for state changes of all components within the circuit.  Usually, this means that stateful elements internally all change state on the same clock event, like a positive edge.  For the rest of this list, I'll assume the positive edge is the event of interest.
- The circuit has a reset line that sets it to a known state when the reset is asserted at the positive clock edge.
- All inputs to the circuit are constrained to only change on the positive clock edge.
- All outputs from the circuit are constrained to only change on the positive clock edges.

Diagrammatically, this means a Synchronous circuit looks like this:

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

which in contrast to the `Circuit` diagram, has a few features that are evident diagrammatically.

- There is a `ClockReset` signal that is fanned out to all of the child elements of the circuit and is provided to the kernel as well.
- The input and output types are `Digital` not `Timed`, since all data is assumed to be on the same time domain as the clock.  This results in a significantly simpler syntax and less mental burden when understanding the circuit operation.

In this section, we will cover this diagram and the underlying traits in detail, and contrast them to `Circuit` as we go.  The goal is to outline how a `Synchronous` circuit differs from `Circuit`, and where simplifications arise as a result.

The design process is the same as `Circuit`.

- Decide the inputs and outputs of your circuit.
- Select the internal synchronous subcircuits that you need
- Write the kernel function
- Validate and test


```admonish note
Why not make everything `Synchronous`?  Because the Real World is not synchronous, and you will inevitably have to deal with inputs and outputs that are not synchronized to any one clock.  Even if your entire design runs off a single clock and all of the inputs and outputs are synchronized to that clock, eventually you will have to interface to something as simple as a switch or button that doesn't play by your clock's rules.  If you ignore that fact, you can end up with circuits that mostly work or stop working.

Why not make everything `Circuit` or asynchronous?  Because asynchronous circuits are complicated and you have to be very careful about making sure that signals don't cross from one time domain to another without special purpose circuitry that "synchronizes" those changes.  In general, I try to keep as much of my design work synchronous as possible, and then only worry about asynchronous `Circuit` behavior at the top level when the different clock regimes interact or we need to talk to the outside world.
```