# Time

There is no inherent notion of time in RHDL.  If you re-examine the definition of the `Circuit` and `Synchronous` traits, you will find no reference to time.  This is because time is not really meaningfull within the operation of a circuit as far as RHDL is concerned.  There is _causality_ in which event `E1` "preceeds" `E2`, and that causality is necessary to model state changes or ensure that invariants are met.  Similarly, when a clock edge changes, the _when_ of that edge change is irrelevant.  The event of it changing is what matters, not the when.  

This lack of time is reflected in the trait definitions.  Here is a summary of the `Circuit` trait definition:

```rust
pub trait Circuit: 'static + CircuitIO + Sized {
    type S: Clone + PartialEq;

    fn init(&self) -> Self::S;
    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O;
    // snip
}
```

Note that there is no notion of "time" here.  It is true that the input is constrained such that `I: Timed` from the `CircuitIO` trait:

```rust
pub trait CircuitIO: 'static + CircuitDQ {
    type I: Timed;
    type O: Timed;
    // snip
}
```

but this is simply a marker trait that refers to the input belonging to some class of signals that "belong to the same timing domain".  Indeed, `Timed` carries no additional runtime information, it is simply a marker trait on top of `Digital`.

Similarly, if we look at the `Synchronous` trait:

```rust
pub trait Synchronous: 'static + Sized + SynchronousIO {
    type S: PartialEq + Clone;
    fn init(&self) -> Self::S;
    fn sim(&self, clock_reset: ClockReset, input: Self::I, state: &mut Self::S) -> Self::O;
    // snip
}
```

there is no notion of time here either.  Simulation via the `sim` method is only involved with the _values_ of the `ClockReset` and input signals presented to it (as well as the current state).  There is no reference to the current absolute time, or when those signals have changed relative to each other.

```admonish note
RHDL does not currently concern itself with physical timing - the process of determining how long signals will take to traverse a design.  Physical timing analysis requires detailed knowledge about the target hardware, and is assumed to be the responsibility of the toolchain you are using to convert your design into a valid representation of the physical hardware.  At the physical timing layer, time in absolute units is critical for ensuring correct operation of your circuit.
```

There is an area in which time becomes helpful, and that is in [tracing](tracing.md).  It is easier to visualize information flowing through a circuit when there is a notion of time imposed (however artificial) on the events.  For example in the Xor gate [example](../xor_gate/making_trace_files.md), we space the inputs a uniform 100 arbitrary units apart so they can be easily seen on the resulting trace file:

![XorGat Simulation](../img/xor.svg)

There is nothing inherently fundamental about the uniform spacing here.  You could have the changes spaced randomly or in whatever way you like.  The simulation code doesn't even use the time.  Only the trace (which is a visualization tool) will react.

For the purposes of attaching time to a value, we have a `TimedSample` struct in RHDL.  It is _very_ simple:

```rust
/// A sample of a digital value at a specific time.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub struct TimedSample<T: Digital> {
    /// The digital value being sampled.
    pub value: T,
    /// The time at which the sample was taken.
    pub time: u64,
}
```

Time is modelled as a `u64`, and the units are intentionally arbitrary.  If you like to think in picoseconds or nanoseconds, you can easily do so.  But using a unit that is something like "1/200th of a clock period" is actually more convenient.  

In any case, when using iterators to drive the simulation of an `impl Circuit` or an `impl Synchronous`, we assume that the inputs and outputs are `TimedSample`.  While this isn't strictly required, it makes it possible to transparently support tracing and visualization, as well as export simulations to test benches and third party tools.  

