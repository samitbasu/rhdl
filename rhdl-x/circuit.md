The missing piece is for the case that you want to define your own D and Q structs.
There are reasons why you would want to do this.  For example, constants do not require
a D signal.  So suppose the end user writes their own D and Q structs.  The place 
where this fails to work is in the `sim` method.   In the `sim` method, the 
simulation loop needs to know how to extract a given input from the struct.  My first
thought is that the `sim` method use `.into()`.   So

trait CircuitDQ {
    type D = MyD;
    type Q = MyQ;
}

trait Circuit : CircuitDQ {

    type AutoD = (c0::I, ... )
    type AutoQ = (c0::O, ... )

    fn sim() {
        let MyQ : Self::Q = autoq.into();
        let (output, MyD) = UPDATE(inputs, MyQ)
        let child_outputs: AutoD = MyD.into();
    }

}

This would allow you to (for example) have a single clock element in the D struct and 
share it across all of the child circuits.  That's neat.  But it means you need to write
a nasty function like:

impl From<MyQ> for AutoQ {
    // What goes here?
}

And

impl From<AutoD> for MyD {
    // What goes here?
}

The automatically generated versions of these _could_ be written on the assumption that
field names are the same.

impl From<MyQ> for AutoQ {
    fn from(val: MyQ) -> AutoQ {
        (val.c0, val.c1, ...) // Assumes each field is present
    }
}

impl From<AutoD> for MyD {
    fn from(val: AutoD) -> MyD {
        MyD {
            c0: val.0,
            c1: val.1,
            ...
        }
    }
}

You _could_ then opt out of using these if you want to customize the behavior somehow.

This at least has the benefit of not writing the D and Q structs for the end user.  Which
seems kind of wrong somehow.

BUT - that means that in the HDL generation, additional code needs to be generated to map
the child inputs/outputs to the provided D/Q structs.  That is definitely not going to work.


#A New Idea!


One idea is to introduce circuit combinations.  We need 3 kinds.  Parallel, Series, and Feedback.
The existing Circuit trait covers all three.  But if we break it down, we can probably handle
the D/Q issue more eloquently (although more verbosely).

Here is the existing Circuit trait:

```rust
pub trait CircuitIO: 'static + Sized + Clone {
    type I: Digital;
    type O: Digital;
}

pub trait Circuit: 'static + Sized + Clone + CircuitIO {
    type D: Digital;
    type Q: Digital;

    // auto derived as the sum of NumZ of the children
    type Z: Tristate;

    type Update: DigitalFn;

    const UPDATE: CircuitUpdateFn<Self> = |_, _| (Default::default(), Default::default());

    // State for simulation - auto derived
    type S: Default + PartialEq + Clone;

    // Simulation update - auto derived
    fn sim(&self, input: Self::I, state: &mut Self::S, io: &mut Self::Z) -> Self::O;

    fn init_state(&self) -> Self::S {
        Default::default()
    }

    // auto derived
    fn name(&self) -> &'static str;

    // auto derived
    fn descriptor(&self) -> CircuitDescriptor;

    // auto derived
    fn as_hdl(&self, kind: HDLKind) -> anyhow::Result<HDLDescriptor>;

    // auto derived
    // First is 0, then 0 + c0::NumZ, then 0 + c0::NumZ + c1::NumZ, etc
    fn z_offsets() -> impl Iterator<Item = usize> {
        std::iter::once(0)
    }
}
```


Now suppose we remove the feedback from the circuit trait.  We then do not
know the form of the UPDATE kernel.  So that must go.  State can still
be preserved, and the HDL func should also (as well as the descriptors).

Let's ignore tristate for now.  It is a real issue...

So the Circuit trait becomes:

```rust
pub trait CircuitIO: 'static + Sized + Clone {
    type I: Digital;
    type O: Digital;
}

pub trait Circuit: 'static + Sized + Clone + CircuitIO {
    // auto derived as the sum of NumZ of the children
    type Z: Tristate;

    // State for simulation - auto derived
    type S: Default + PartialEq + Clone;

    // Simulation update - auto derived
    fn sim(&self, input: Self::I, state: &mut Self::S, io: &mut Self::Z) -> Self::O;

    fn init_state(&self) -> Self::S {
        Default::default()
    }

    // auto derived
    fn name(&self) -> &'static str;

    // auto derived
    fn descriptor(&self) -> CircuitDescriptor;

    // auto derived
    fn as_hdl(&self, kind: HDLKind) -> anyhow::Result<HDLDescriptor>;

    // auto derived
    // First is 0, then 0 + c0::NumZ, then 0 + c0::NumZ + c1::NumZ, etc
    fn z_offsets() -> impl Iterator<Item = usize> {
        std::iter::once(0)
    }
}
```

This all looks quite reasonable.  We may be heading to the possibility of `sim` being defined
via normal functions instead of code generation...

Now consider the simple case of composing 2 circuits in series.  This _should_ look something like

```rust
type Foo = SeriesCircuit<C0, F, C1>;
```

How about:

```rust
struct SeriesCircuit<C0 : Circuit, F : Kernel<C0::O, C1:I>, C1 : Circuit> {
    c0: C0,
    c1: C1,
}

impl<C0: Circuit, F: Kernel<C0::O, C1::I>, C1: Circuit> CircuitIO for SeriesCircuit<C0, F, C1> {
    type I = C0::I;
    type O = C1::I;
}


impl<C0: Circuit, F: Kernel<C0::O, C1::I>, C1: Circuit> Circuit for SeriesCircuit<C0, F, C1> {
    type Z = (C0::Z, C1::Z);
    type S = (C0::S, C1::S);

    fn sim(&self, input: Self::I, state: &mut Self::S, iobuf: &mut Self::Z) -> Self::O {
        let o0 = self.c0.sim(input, state.0, iobuf.0);
        let i1 = F::UPDATE(o0);
        self.c1.sim(i1, state.1, iobuf.1);
    }
}
```

This looks pretty clean.  The rest of the internals should be fine.  What about parallel?

```rust


struct ParallelCircuit<
   I: Digital, 
   Fin: Kernel<I, (C0::I, C1::I)>, 
   C0: Circuit, 
   C1: Circuit, 
   FnOut: Kernel<(C0::O, C0::O), O>, 
   O: Digital>
> {
    c0: C0,
    c1: C1,
}
