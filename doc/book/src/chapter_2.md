# Chapter 2

## Setting Up

```shell,rhdl-silent
# Remove the foo directory to get a clean slate
rm -rf xor
```

Let's start by creating a new Rust project.  We want a library, and we need the `rhdl` dependency.  `RHDL` uses `miette` to provide error reporting, so we will add that as well.

```shell,rhdl
cargo new --lib xor
cd xor
cargo add --path ~samitbasu/Devel/rhdl/crates/rhdl rhdl
cargo add --dev miette
```

In the `src` directory, we will start by importing the prelude for `RHDL`, which brings a lot of useful signals into scope. 

```rust,write:xor/src/lib.rs
use rhdl::prelude::*;
```

So far, so good.

```shell,rhdl:xor
cargo check -q
```

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

The function needs to be `pub` For Reasons.  Ok, so we now have these `Signal` things, and need to compute the XOR function.  You can't do much with a `Signal` type itself, but it's just a wrapper, and you can get at the underlying value with the `.val()` method.  There is also a type-inferred constructor function named `signal` to build a `Signal` out of a value.  So most of the kernel is just unwrapping and rewrapping the values.  

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

## Basic Testing

We probably want to test all possible inputs of our `XorGate`, and since there are only four inputs, it shouldn't be too hard.  We can start by testing our kernel itself.  Just as a plain Rust function (which it still is...)

```shell,rhdl:xor
mkdir tests
```

```rust,write:xor/tests/test_inputs.rs
use rhdl::prelude::*;
use xor::*;

#[test]
fn test_all_inputs() {
    let inputs = [(false, false), (false, true), (true, false), (true, true)];
    let outputs = [false, true, true, false];
    inputs.iter().zip(outputs.iter()).for_each(|(inp, outp)| {
        let (y, _) = xor_gate(signal(*inp), ());
        assert_eq!(y.val(), *outp);
    });
}
```

```shell,rhdl:xor
cargo build -q
cargo test 
```

Ok - that was easy enough.  But that just tests that our logic was correct, right?  What about testing more of the things?  How do I know the generated hardware will work as intended?  And what does the generated hardware look like, anyway?  The simpleset way to get a view on the generated HDL is to use the `.hdl` method on any struct that `impl Circuit`.  The result can be converted into a module and then a string.   The following test does exactly that.

```rust,write:xor/tests/show_verilog.rs
use rhdl::prelude::*;

#[test]
fn show_verilog() -> miette::Result<()> {
     let gate = xor::XorGate;
     let hdl = gate.hdl("xor_gate")?.as_module();
     eprintln!("{hdl}");
     Ok(())
}
```

```shell,rhdl:xor
cargo build -q
cargo test --test show_verilog -- --nocapture
```

While not required, it is often handy to check that the output of an HDL generation step has not changed from the last time you reviewed or tested it.  As such, a crate like [expect-test](https://github.com/rust-analyzer/expect-test) can be used to check that the output is still correct.  We can add it as a `dev` dependency to our project

```shell,rhdl:xor
cargo add --dev expect-test
```

A test using `expect-test` can write the expected Verilog code to a file and, then verify it later.

```rust,write:xor/tests/expect_verilog.rs
use rhdl::prelude::*;

#[test]
fn test_verilog_output() -> miette::Result<()> {
     let gate = xor::XorGate;
     let hdl = gate.hdl("xor_gate")?.as_module();
     let expect = expect_test::expect_file!["xor.v.expect"];
     expect.assert_eq(&hdl.to_string());
     Ok(())
}
```

You can run the test with an `UPDATE_EXPECT=1` to get the expected output to be written to a file.
```shell,rhdl:xor
cargo build -q
UPDATE_EXPECT=1 cargo test --test expect_verilog
cat tests/xor.v.expect
```

Then in the future, you can run the test, and it will compare the generated code against the template file stored.

```shell,rhdl:xor
cargo test --test expect_verilog
```

## Iterator Based Testing

