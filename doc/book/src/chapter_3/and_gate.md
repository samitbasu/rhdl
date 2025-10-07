# The And Gate

```shell,rhdl-silent
# Remove the half adder directory to get a clean slate
rm -rf half
```

Let's start by creating a new Rust project.  This will again be a library.  We will need the `rhdl` dependency. 

```shell,rhdl
cargo new --lib half
cd half
cargo add --path ~samitbasu/Devel/rhdl/crates/rhdl rhdl
cargo add --dev miette
```

For simplicity, I will add the completed `xor` gate from the previous chapter, without the test fixtures.  

```rust,write:half/src/xor.rs
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

And here is the equivalent for an And gate.  I don't recommend building actual designs this way - it is very very low level, but it is illustrative.

```rust,write:half/src/and.rs
use rhdl::prelude::*;

#[derive(Circuit, Clone)]
pub struct AndGate;

impl CircuitDQ for AndGate {
    type D = (); 
    type Q = (); 
}

impl CircuitIO for AndGate {
    type I = Signal<(bool, bool), Red>;
    type O = Signal<bool, Red>;
    type Kernel = and_gate;
}

#[kernel]
pub fn and_gate(i: Signal<(bool, bool), Red>, _q: ()) -> (Signal<bool, Red>, ()) {
    let (a, b) = i.val();
    let c = a & b;
    (signal(c), ())
}
```

We include both gates into our top level `lib.rs` like this:

```rust,write:half/src/lib.rs
mod xor;
mod and;
```

```shell,rhdl-silent:half
cargo check -q
```

We check that it compiles

```shell,rhdl:half
cargo check
```

Given that our And gate is trivial by construction, we won't bother testing it, but you may want to repeat the exercises from the `Xor Gate` chapter yourself.  Let's move on to building our half-adder.