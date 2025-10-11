## The Gate

Next comes our `XorGate`, which has no internals,  so the `struct` that describes it is a unit.

```rust,write:xor/src/lib.rs
use rhdl::prelude::*;

pub struct XorGate;
```

We need to provide the definitions of `I, O, D, Q` as described previously.  These are done by the `CircuitIO` and `CircuitDQ` traits.  The `D` and `Q` types are easy.  There is no internal structure so they are both empty.

```rust
use rhdl::prelude::*;

pub struct XorGate;

impl CircuitDQ for XorGate {
     type D = (); 
     type Q = (); 
}
```

For the input and output types, we need types that `impl Timed`.  There is a subtlety here that involves with how asynchronous signals are handled in RHDL.  We will return to this later.  For now, we need to understand that an XOR gate really needs to manipulate signals that belong to the same time domain (whatever that may be).  In RHDL, time domains are represented by colors, so we pick one (`Red` because its short to type), and indicate that the input of our XOR gate is a pair of 1-bit signals in some time domain, and the output is a single 1-bit signal in the same time domain.  For simplicity, we will use a `(bool, bool)` tuple on the input, and a single `bool` on the output:

```rust
use rhdl::prelude::*;

pub struct XorGate;

impl CircuitDQ for XorGate {
     type D = (); 
     type Q = (); 
}

impl CircuitIO for XorGate {
     type I = Signal<(bool, bool), Red>;
     type O = Signal<bool, Red>;
     type Kernel = xor_gate;    // 👈 doesn't exist yet
}
```

So far, we have described our gate as looking like this:

```badascii
             +-+XorGate+-+       
(bool,bool)  |           | bool  
+----------->|     ?     +------>
             |           |       
             +-----------+       
```
where the time domain has been suppressed on the diagram as implied.  With these `impl` in place,
we can go back and add the `derive` that implements the `Circuit` trait for us:

```rust
use rhdl::prelude::*;

#[derive(Circuit, Clone)] // 👈 new!
pub struct XorGate;

impl CircuitDQ for XorGate {
     type D = (); 
     type Q = (); 
}

impl CircuitIO for XorGate {
     type I = Signal<(bool, bool), Red>;
     type O = Signal<bool, Red>;
     type Kernel = xor_gate;
}
```

The last piece is the kernel itself.  The signature for the kernel is described in the `CircuitIO` trait:

```rust,ignore
type Kernel: DigitalFn + DigitalFn2<A0 = Self::I, A1 = Self::Q, O = (Self::O, Self::D)>;
```

which is an ugly way of saying that `Kernel` has the shape of `fn(I, Q) -> (O, D)`.  So let's write it as
such.

```rust
# use rhdl::prelude::*;
 👇 needed!
pub fn xor_gate(i: Signal<(bool, bool), Red>, q: ()) -> (Signal<bool, Red>, ()) {
     todo!()
}
```

The function needs to be `pub` For Reasons™.  Ok, so we now have these `Signal` things, and need to compute the XOR function.  You can't do much with a `Signal` type itself, but it's just a wrapper, and you can get at the underlying value with the `.val()` method.  There is also a type-inferred constructor function named `signal` to build a `Signal` out of a value.  So most of the kernel is just unwrapping and rewrapping the values.  

```rust
#use rhdl::prelude::*;
pub fn xor_gate(i: Signal<(bool, bool), Red>, q: ()) -> (Signal<bool, Red>, ()) {
     let (a, b) = i.val(); // a and b are both bool
     let c = a ^ b; // Exclusive OR
     (signal(c), ())
}
```

Finally, we need to turn this ordinary Rust function into something synthesizable in hardware, and for that we need the `#[kernel]` attribute.  

```rust
#[kernel] // 👈 new!
pub fn xor_gate(i: Signal<(bool, bool), Red>, q: ()) -> (Signal<bool, Red>, ()) {
     let (a,b) = i.val(); // a and b are both bool
     let c = a ^ b; // Exclusive OR
     (signal(c), ())
}
```

Great!  That may seem like a lot of boiler plate for a lowly `XOR` gate, but remember that we are intentionally adding verbosity here.  We want to signal our intentions with the type system, and that requires extra words.  It will all be worth it when the complexity grows.

So here is our completed `XorGate`:

```rust,write:xor/src/lib.rs
use rhdl::prelude::*;

#[derive(Circuit, Clone)]
pub struct XorGate;

impl CircuitDQ for XorGate {
    type D = (); 
    type Q = (); 
}

impl CircuitIO for XorGate {
    type I = Signal<(bool, bool), Red>;
    type O = Signal<bool, Red>;
    type Kernel = xor_gate;
}

#[kernel]
pub fn xor_gate(i: Signal<(bool, bool), Red>, _q: ()) -> (Signal<bool, Red>, ()) {
    let (a, b) = i.val();
    let c = a ^ b;
    (signal(c), ())
}
```

```shell,rhdl-silent:xor
cargo check -q
```

```shell,rhdl:xor
cargo check
```

It would probably be a good idea to test our circuit, right?  So let's turn to testing.
