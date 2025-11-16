# Clock Pos Edge

In introducing the next extension trait, we need to take a look at one of the details of simulation that are important for synchronous circuits.  _Generally_ when you see a timing diagram for a simulation, you will see something like this:

```badascii
data    +----------+--------------------------------+-----+
           x1      |              x2                |      
        +----------+--------------------------------+-----+
                                                           
                   +---------------+                +-----+
clk                |               |                |      
        +----------+               +----------------+      
```

In general, the accompanying language will be something to the effect that the data "changes on the positive going edge of the clock".  The challenge with _simulating_ something like this is that it makes the clock signal "special".  Suppose, for example that:

- At time `t0 - δ`, the data signal is `x1`, the clock is `low`
- At time `t0`, the data signal transitions to `x2` and the clock transitions to `high`

If these two transitions happen simultaneously, then _the order in which the changes are considered matters_.  In other words, if you process the events in this order:

1. Data transitions
2. Clock transitions

you will interpret the result differently than if you process the events in this order:

1. Clock transitions
2. Data transitions

In some sense, when the two signals transition _simultaneously_ there is no unambiguous way to decide which one transitioned "first".  The problem with the indeterminancy is that we associate the value of data that was "captured" at the rising edge of the clock differently depending on the two interpretations.  If we process the data transition _first_ then when the clock transitions, the data has already gone from `x1 -> x2` and the input to the circuit is `x2`.  If we process the clock transition _first_ then the input to the circuit is `x1`.  

So how do we know which transition event to process first when faced with a list of signals that all changed at time `t0`?  In general, you should probably process the clock first.  But what if there are multiple clocks? And they all transition at the same time?  Do we process all the clocks first? And why isn't this a problem in real life?  

Well, in real life, the solution is simple.  Just make the path the clock takes to reach the circuit slightly shorter than the signal.  This means that the clock edge is arrives at the circuit input _before_ the signal edge, and the diagram looks like this:

```badascii
data     +----------+--------------------------------+-----+
            x1      |              x2                |  x3  
         +----------+--------------------------------+-----+
                +>|δ|<+                          +>|δ|<+    
                  +---------------+                +-------+
clk               | :             |                | :      
         +--------+ :             +----------------+ :      
                                                            
         ^        ^ ^             ^                ^ ^      
         +        + +             +                + +      
                                                            
 data   x1       x1 x2           x2               x2 x3     
 clk     F        T T             F                T T      
```
The introduction of this small time delay disambiguates the order of transitions.  The clock transitions first, and then the data can transition.  You often won't see a `δ` when you look at traces or at diagrams, but it is definitely there.

Simulators use special case logic to detect clock signals, and then try to juggle the event order so that the clock transitions are processed first.  It usually works, but it's not perfect.  RHDL, for example bundles the clock and reset signals together, and that can confuse some simulation engines.  As a result, we take the extra precaution of introducing a delay of `δ = 1` time sample into the simulation.    

To do this easily, there is another extension trait, `ClockPosEdgeExt`, which is described as follows:

```rust
impl<I, Q> ClockPosEdgeExt<Q> for I
where
    I: IntoIterator<Item = ResetOrData<Q>>,
    Q: Digital,
{
    fn clock_pos_edge(self, period: u64) -> ClockPosEdge<Self::IntoIter, Q>;
}
```

Calling `clock_pos_edge(period)` on an iterator that yields items of type `ResetOrData<I>` where `I` is the input type of the circuit, produces a stream of input data that can be used to simulate the circuit.   For example, the following snippet of Rust code:

```rust
(0..4).map(b8).without_reset().clock_pos_edge(10)
```
 
yields the following sequence of values:


| time | clock | reset | value |
|-----|--------|-------|--------|
| 0  | false |  false |  b8(0) |
| 5  | true |  false |  b8(0) | 
| 6  | true |  false |  b8(1) |
| 10 | false |  false |  b8(1) |
| 15 | true |  false |  b8(1) |
| 16 | true |  false |  b8(2) |
| 20 | false |  false |  b8(2) |
| 25 | true |  false |  b8(2) |
| 26 | true |  false |  b8(3) |
| 30 | false |  false |  b8(3) |
| 35 | true |  false |  b8(3) |


Note the "extra" sampels at times `6, 16, 26`.  These are the result of the need to transition the input value 1 time step after the clock changes.  It's a detail, but important, particularly if you plan to drive your simulation yourself. 

```admonish warning
To ensure correct behavior, make sure that all inputs to the circuit change at some time `t` that is strictly greater than the time at which the clock edge transitions. This extra "hold" time is not strictly required by RHDL.  But there are instances in which RHDL generates test benches run through external simulation tools.  Adding the hold of 1 time unit improves the simulation of those test benches.
```

