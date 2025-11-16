# Composing the Half Adder

We are now ready to compose our half adder from an `XorGate` and an `AndGate`.  Referring back to our foundational diagram, we can now begin to define the various types that make up the inputs, outputs, and feedback elements for our circuit.  


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

The input type is still a tuple of booleans `Signal<(bool, bool), Red>`, but the output signals now need to be differentiated.  One is the sum and the other is the carry value.  Using a tuple for the output would be confusing, since it would be unclear which bit in the output corresponded to which.  Ideally, we want the circuit too look like this:

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

The inputs `a` and `b` can be interchanged, but the outputs cannot.  So in this case, we will create a struct to hold the output of the circuit, and name the outputs.  All synthesizable data structures in RHDL must `impl Digital`, the trait that allows them to be transformed into bitpatterns, and that provides a run time type system for describing the shape of the data.  In this case, we will define a struct to carry the outputs of the circuit, and use the `#[derive(Digital)]` trait to add the needed methods.  We also need to indicate that the output struct carries timed data (meaning that each element belongs to some timing domain).  So we also need to `#[derive(Timed)]`.  Finally, for technical reasons, you must also derive `PartialEq` for your custom data types.  Here is what our half-adder output looks like:

```rust
#[derive(Digital, Copy, Clone, Timed, PartialEq)]
pub struct Outputs {
    pub sum: Signal<bool, Red>,
    pub carry: Signal<bool, Red>,
}
```

Of course, the name `Outputs` is arbitrary.  In general, naming inputs and outputs will make working with circuit components easier than using tuples.  And the names of the inputs and outputs will come in handy later when writing kernel functions.  Next, we declare the actual half adder itself.  It will contain two child circuits, an `XorGate` and an `AndGate`.  

```rust
#[derive(Circuit, Clone)]
pub struct HalfAdder {
    xor: xor::XorGate,
    and: and::AndGate,
}
```

Thus, composition in RHDL is accomplished structurally.  If you want to build a complex circuit, you compose simpler subcircuits into a `struct` and then `#[derive(Circuit)]`.  It is really that simple.  

We need a way to construct the circuit.  Since a half adder has no real configuration, `Default` is a good choice:

```rust
impl Default for HalfAdder {
    fn default() -> Self {
        Self {
            xor: xor::XorGate,
            and: and::AndGate,
        }
    }
}
```

Constructors can do arbitrary complicated runtime Rust things, including table construction, web-calls, etc. 

Now, referring back to the foundational diagram, we see that two additional types `D` and `Q` are required.  The `D` type must be defined exactly as follows:

```rust
#[derive(Digital, Copy, Clone, Timed, PartialEq)]
pub struct D {
    xor: <xor::XorGate as CircuitIO>::I,
    and: <and::AndGate as CircuitIO>::I,
}
```

and the `Q` type must be defined as

```rust
#[derive(Digital, Copy, Clone, Timed, PartialEq)]
pub struct Q {
    xor: <xor::XorGate as CircuitIO>::O,
    and: <and::AndGate as CircuitIO>::O,
}
```

These types must follow the pattern of having

- Each field name in `D` and `Q` must map exactly to a field in the parent circuit (the HalfAdder)
- The types of the fields in `D` must be the input types for the subcircuits
- The types of the fields in `Q` must be the output types for the subcircuits

There is a handy way to autoderive these, but for now, we will provide explicit definitions.  We can now provide the trait definitions for `CircuitDQ` and `CircuitIO`, with the kernel still missing:

```rust
impl CircuitDQ for HalfAdder {
    type D = D;
    type Q = Q;
}

impl CircuitIO for HalfAdder {
    type I = Signal<(bool, bool), Red>;
    type O = Outputs;
    type Kernel = half_adder; // 👈 doesn't exist 
}
```

At this point, the total contents of the half adder `lib.rs` are as follows:

```rust,write:half/src/lib.rs
use rhdl::prelude::*;

mod and;
mod xor;

pub use xor::XorGate;
pub use xor::xor_gate;

#[derive(Digital, Copy, Clone, Timed, PartialEq)]
pub struct Outputs {
    pub sum: Signal<bool, Red>,
    pub carry: Signal<bool, Red>,
}

#[derive(Circuit, Clone)]
pub struct HalfAdder {
    xor: xor::XorGate,
    and: and::AndGate,
}

impl Default for HalfAdder {
    fn default() -> Self {
        Self {
            xor: xor::XorGate,
            and: and::AndGate,
        }
    }
}

impl CircuitIO for HalfAdder {
    type I = Signal<(bool, bool), Red>;
    type O = Outputs;
    type Kernel = half_adder;
}

#[derive(Digital, Copy, Clone, Timed, PartialEq)]
pub struct D {
    xor: <xor::XorGate as CircuitIO>::I,
    and: <and::AndGate as CircuitIO>::I,
}

#[derive(Digital, Copy, Clone, Timed, PartialEq)]
pub struct Q {
    xor: <xor::XorGate as CircuitIO>::O,
    and: <and::AndGate as CircuitIO>::O,
}

impl CircuitDQ for HalfAdder {
    type D = D;
    type Q = Q;
}
```

The last step is to write the kernel function.  In this case, the signature of the kernel is

```rust
//                              👇 Input type
pub fn half_adder(i: Signal<(bool,bool), Red>, q: Q) -> (Outputs, D) {
    todo!()
}
```

The kernel function must:

- given the current inputs `i: Signal<(bool,bool), Red>`
- given the current outputs of internal components `q: Q`

compute the circuit output `O` and the inputs to the internal components `d: D`.  In this case, the kernel function is fairly straightforward.  The subcircuits are simply to be fed copies of the input signals, and the output is collected from their computed outputs:

```rust
#[kernel]
pub fn half_adder(i: Signal<(bool, bool), Red>, q: Q) -> (Outputs, D) {
    // D is the set of inputs for the internal components
    let d = D {
        xor: i, 
        and: i, // 👈 Digital : Copy, so no cloning needed
    };
    // Q is the output of those internal components
    let o = Outputs {
        sum: q.xor,
        carry: q.and,
    };
    (o, d)
}
```

The output is computed by taking the sum as the output of the `xor` gate, which is `q.xor`, and the carry as the output of the `and` gate, which is `q.and`.  The complete circuit defition is thus:

```rust,write:half/src/lib.rs
use rhdl::prelude::*;

mod and;
mod xor;

pub use xor::XorGate;
pub use xor::xor_gate;

#[derive(Digital, Copy, Clone, Timed, PartialEq)]
pub struct Outputs {
    pub sum: Signal<bool, Red>,
    pub carry: Signal<bool, Red>,
}

#[derive(Circuit, Clone)]
pub struct HalfAdder {
    xor: xor::XorGate,
    and: and::AndGate,
}

impl Default for HalfAdder {
    fn default() -> Self {
        Self {
            xor: xor::XorGate,
            and: and::AndGate,
        }
    }
}

impl CircuitIO for HalfAdder {
    type I = Signal<(bool, bool), Red>;
    type O = Outputs;
    type Kernel = half_adder;
}

#[derive(Digital, Copy, Clone, Timed, PartialEq)]
pub struct D {
    xor: <xor::XorGate as CircuitIO>::I,
    and: <and::AndGate as CircuitIO>::I,
}

#[derive(Digital, Copy, Clone, Timed, PartialEq)]
pub struct Q {
    xor: <xor::XorGate as CircuitIO>::O,
    and: <and::AndGate as CircuitIO>::O,
}

impl CircuitDQ for HalfAdder {
    type D = D;
    type Q = Q;
}

#[kernel]
pub fn half_adder(i: Signal<(bool,bool), Red>, q: Q) -> (Outputs, D) {
    let d = D {
        xor: i,
        and: i,
    };
    let o = Outputs {
        sum: q.xor,
        carry: q.and,
    };
    (o, d)
}
```

We can verify that this does indeed compile

```shell,rhdl:half
cargo check
```
