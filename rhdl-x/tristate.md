For a bidirectional bus, the bidirectional signals could be handled in a couple of ways.

1. They could be "pattern matched" as being the same bits in both the input and output
of a kernel.  For example,

```rust
fn update(in: I, q: Q) -> (O, D)
```

Suppose that `I` and `O` both have a field named `foo` with type `T`.  Then we could
infer that this is an inout signal...

This seems like a terrible idea.  Since in most cases, this is probably not what
the end user wants to happen...  Yuck.

2. The inout signals could be special cased.

We can add an inout type to the struct.  Let's call this type Z.  

```rust
trait Circuit {
  type Z : Digital
}
```

If this type is non-empty, then we need a second function to connect it to
the Z types of the internal components.  For example, suppose

```rust
struct BiDi {
    left: b8;
    right: b8;
}
```

This is a 16-bit bidirectional bus with two logical parts, each of 8 bits.
Now we set

```rust
type Z = BiDi
```

Which means we need a new function to take this and do something with it.  But 
we cannot generally compute or do anything else with these values. 


What we need is to define for each child a path from Z to the child's Z.  We want
that path to be defined simply.  One way is to define a new auto type

```rust
struct CircuitC {
    child_0: Child0::Z,
    child_1: Child1::Z,
    ...
    child_n: ChildN::Z,
};
```

And then provide an update function to bridge the two.

```rust
#[kernel]
pub fn link(top: Self::Z) -> Self::C;
```

This link function would then partition the bus amongst
the children of the current module.  An additional check
pass would have to reduce this to pure paths.





```rust
#[derive(Clone, Debug, Default, LogicInterface)]
#[join = "BidiBusD"]
pub struct BidiBusM<T: Synth> {
    pub sig_inout: Signal<InOut, T>,
    pub sig_empty: Signal<In, Bit>,
    pub sig_full: Signal<In, Bit>,
    pub sig_not_read: Signal<Out, Bit>,
    pub sig_not_write: Signal<Out, Bit>,
    pub sig_master: Signal<Out, Bit>,
}

#[derive(Clone, Debug, Default, LogicInterface)]
#[join = "BidiBusM"]
pub struct BidiBusD<T: Synth> {
    pub sig_inout: Signal<InOut, T>,
    pub sig_empty: Signal<Out, Bit>,
    pub sig_full: Signal<Out, Bit>,
    pub sig_not_read: Signal<In, Bit>,
    pub sig_not_write: Signal<In, Bit>,
    pub sig_master: Signal<In, Bit>,
}

#[derive(LogicBlock, Default)]
pub struct BidiMaster<T: Synth, const N: usize, const NP1: usize> {
    pub bus: BidiBusM<T>,
    pub bus_clock: Signal<In, Clock>,
    bus_buffer: TristateBuffer<T>,
    fifo_to_bus: AsynchronousFIFO<T, N, NP1, 1>,
    fifo_from_bus: AsynchronousFIFO<T, N, NP1, 1>,
    pub data: FifoBus<T>,
    pub data_clock: Signal<In, Clock>,
    state: DFF<BidiState>,
    can_send_to_bus: Signal<Local, Bit>,
    can_read_from_bus: Signal<Local, Bit>,
}
```


The BITS option.  One option is to require tristate busses be expressed as pure Bits.  This isn't quite ideal,
but it allows you to relax additional requirements on the mask bits.  So in this case, 