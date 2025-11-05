# Inputs and Outputs

Recall from the foundational diagram that the circuit inputs and outputs are encapsulated into two types `I` and `O`:


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
       | q.child_1 +> +----+o        child_1      i|<----+ <+ d.child_1 |        
       |              |    +-----------------------+     |              |        
       |              |                                  |              |        
       |              |    +-----------------------+     |              |        
       | q.child_2 +> +----+o        child_2      i|<----+ <+ d.child_2 |        
       |                   +-----------------------+                    |        
       |                                                                |        
       +----------------------------------------------------------------+        
```

The way we specify these types is via the `CircuitIO` trait.  Here is the definition of that trait:

```rust
pub trait CircuitIO: 'static + CircuitDQ {
    type I: Timed;
    type O: Timed;
    type Kernel: DigitalFn + DigitalFn2<A0 = Self::I, A1 = Self::Q, O = (Self::O, Self::D)>;
}
```

Ignoring the `Kernel` associated type for the moment, we see that `I: Timed` and `O: Timed`.  In general, a `Timed` type is either:

- `Signal<T, D>` where `D: Domain` and `T: Digital` - this represents a signal of type `T` that changes in accordance with the time domain `D`.
- `()` - the empty type is also `Timed`
- Some tuple of `: Timed` types or an array `[T; N]` of them

See the [Timed Types](../timed/summary.md) section for more information about the `Timed` trait.

The `CircuitIO` trait cannot be auto-derived by RHDL, since only you know what the input and output types of the circuit are.  For example in the [Half Adder](../half_adder/half_adder.md), the input is a pair of boolean signals that result in a `sum` and `carry` signal.  Here are the types involved:

```rust
#[derive(Digital, Copy, Clone, Timed, PartialEq)]
pub struct Outputs {
    pub sum: Signal<bool, Red>,
    pub carry: Signal<bool, Red>,
}

impl CircuitIO for HalfAdder {
    type I = Signal<(bool, bool), Red>;
    type O = Outputs;
    type Kernel = half_adder; // 👈 function `half_adder` is decorated with #[kernel]
}
```

The [CircuitIO] trait also links the compute kernel to the circuit.  This is done by annotating a function with the appropriate signature with a `#[kernel]` and then providing it's name _as a type_ to the `CircuitIO` trait as the associated type `Kernel`.  For example, in the most trivial example of an [Xor Gate](../xor_gate/summary.md), the kernel is 

```rust
#use rhdl::prelude::*;
pub fn xor_gate(i: Signal<(bool, bool), Red>, q: ()) -> (Signal<bool, Red>, ()) {
     let (a, b) = i.val(); // a and b are both bool
     let c = a ^ b; // Exclusive OR
     (signal(c), ())
}
```

```admonish note
It is a feature of RHDL that the compute kernel used in the `Circuit` and `Synchronous` traits is just a normal Rust function.  You can (and should) write it without the `#[kernel]` annotation at first, and only when you are sure it is correct add the annotation to make it synthesizable.  I usually work this way.  Note that there is a `NoCircuitKernel` type that you can use as a placeholder to make `rust` satisfied with the `impl CircuitIO` block while you work on your kernel function.
```

The signature of the kernel (for a `Circuit`) is always

```badascii
                             internal              internal 
                             feedback              feedback
                                +                      +   
                                v                      v   
                                                           
pub fn kernel(i: Self::I, q: Self::Q) -> (Self::O, Self::D)
                                                           
                   ^                         ^             
                   +                         +             
                circuit                   circuit          
                inputs                    outputs          
```


```admonish note
If it strikes you as odd that we are assigning the kernel (which is a function _value_) to an associated _type_, then you have noticed one of the various tricks that RHDL employs to make this all work.  In particular, when you decorate a function `fn foo` with `#[kernel]`, then the resulting macro defines a _type_ named `foo` that contains the meta data needed to manipulate the function as a synthesizable chunk of code, as well as a pointer back to the actual function value.  If there is interest, I can provide some more details on how this works, but suffice to say, we are using the same name for both the value (which you provide) and the type (which RHDL provides).
```

