# Timing Estimation

## Baseline Performance

While we have a design that works, it is not particularly efficient.  If you were to diagram out our ones counter, it would look something like this:

```badascii
 b0    +---+                  
+----->|   |    +---+         
 b1    | + +--->|   |    +---+
+----->|   |    | + +--->|   |
 b2    +---+    |   |    |   |
+-------------->|   |    | + |
 b3             +---+    |   |
+----------------------->|   |
                         +---+
            .                 
            .                 
            .                 
```

where each successive bit is added to the output of the previous one.  This deep chain can create a design that runs slowly, since the change of the signal `b0` has a long way to propagate before it reaches the final `sum` output.  To see that this is the case, we can use another method of the `IceStorm` toolchain.  This one takes a design and computes the longest path through it without actually building a binary or flashing the design to an FPGA.  It gives us a simple way to estimate the timing propagation through a RHDL design.  We will put it into a test case for ease of access later.

```rust,write:ones/tests/test_base_timing.rs
#[test]
fn test_base_timing() -> miette::Result<()> {
    let uut = ones::OneCounter {};
    let timing = rhdl_toolchains::icestorm::IceStorm::new("hx8k", "cb132", "build").time(uut)?;
    eprintln!("Timing: {timing:?}");
    eprintln!(
        "Total delay: {} nsec",
        timing.logic_delay + timing.routing_delay
    );
    Ok(())
}
```

Running this test shows that the time to propogate through our circuit is roughly 11 nanoseconds:

```shell
cargo nextest run base_timing --no-capture
```

While currently, that doesn't matter, in the future, we may want to put this design into a clocked circuit.  With an 11 nanosecond delay, it means that we wont be able to run that clocked circuit particularly quickly.  Furthermore, there isn't anything an optimizer can really do at this point.  The design has already been optimized by `yosys`.  To improve the performance, we need to rethink how we count the ones in the first place.  RHDL gives us tools to do hardware design, but does not eliminate the need to think!  Ideally, it just makes it easier to translate our ideas into working code with fewer surprises.

## Divide and Conquer

The simplest strategy to flatten the design is to divide and conquer.  Calculating the sum of ones is parallelizable over the bits in the input vector.  So we should be able to simply split the sum into two parts:

```badascii
          +-----+                 
  b0..b3  | cnt |                 
+-------->| one +---+   +-----+   
          |     |   |   |     |   
          +-----+   +-->|     |   
                        | sum +-->
          +-----+   +-->|     |   
  b4..b7  | cnt |   |   |     |   
+-------->| one +---+   +-----+   
          |     |                 
          +-----+                 
```

Each of the 4-bit one-counters will have half the tree depth of the 8-bit one-counter, and should thus give a better performance.  So we will try to split up our ones-counter into a pair of smaller counters and then add the results.

## Pure Functions

To make it easier to compare designs, we will first move the existing logic into it's own function, and make it generic over the size of the bitvector being summed.  Let us put the following function into `helper.rs` and add the `pub mod helper` line to `lib.rs` so that we include it.

```rust,write:ones/src/helper.rs
use rhdl::prelude::*;

#[kernel]
pub fn count_ones<N: BitWidth, M: BitWidth>(x: Bits<N>) -> Bits<M> {
    let mut count = bits(0);
    for i in 0..N::BITS {
        if x & (1 << i) != 0 {
            count += 1;
        }
    }
    count
}
```

There are some new things introduced here.  First, we have a `#[kernel]` that has a different type signature than our `fn(I,Q) -> (O, D)`.  This is because you can generally write any pure function as a `kernel` by adding the `#[kernel]` annotation to the signature.  The second is that we have made the function generic over the bitwidth of the input and output bitvectors.  This would allow us to use it parametrically throughout our designs.  Because it is just a plain Rust function, we can still test it with a simple test:

```rust,write:ones/tests/test_helper.rs
use rhdl::prelude::*;
#[test]
fn test_count_ones() {
    assert_eq!(ones::helper::count_ones::<U8, U4>(b8(0b10110010)), b4(4));
}
```

Running `cargo nextest` shows us this test passes as well.

```shell,rhdl:ones
cargo nextest run test_count_ones
```

We can in fact, rewrite our top level kernel to just call the helper function.  With that change, we have the following for `lib.rs`:

```rust,write:ones/src/lib.rs
use rhdl::prelude::*;

pub mod helper;

#[derive(Circuit, Clone)]
pub struct OneCounter {}

impl CircuitIO for OneCounter {
    type I = Signal<b8, Red>;
    type O = Signal<b4, Red>;
    type Kernel = one_counter;
}

impl CircuitDQ for OneCounter {
    type D = ();
    type Q = ();
}

#[kernel]
pub fn one_counter(input: Signal<b8, Red>, _q: ()) -> (Signal<b4, Red>, ()) {
    let input = input.val();
    let count = helper::count_ones::<U8, U4>(input);
    (signal(count), ())
}
```

Let's rerun the tests to see that they all pass.

```shell,rhdl:ones
cargo nextest run
```

Now we can create a new module where the circuit uses subdivision to compute the number of ones.

```rust,write:ones/src/divided.rs
use rhdl::prelude::*;

#[derive(Circuit, Clone)]
pub struct OneCounterDivided {}

impl CircuitIO for OneCounterDivided {
    type I = Signal<b8, Red>;
    type O = Signal<b4, Red>;
    type Kernel = one_counter_divided;
}

impl CircuitDQ for OneCounterDivided {
    type D = ();
    type Q = ();
}

#[kernel]
pub fn one_counter_divided(input: Signal<b8, Red>, _q: ()) -> (Signal<b4, Red>, ()) {
    let input = input.val();
    let lsbs = input.resize::<U4>();
    let msbs = (input >> 4).resize::<U4>();
    let count =
        crate::helper::count_ones::<U4, U4>(lsbs) + crate::helper::count_ones::<U4, U4>(msbs);
    (signal(count), ())
}
```

We add the relevant line to the `lib.rs`

```rust,write:ones/src/lib.rs
use rhdl::prelude::*;

pub mod divided; // 👈 new!
pub mod helper;

#[derive(Circuit, Clone)]
pub struct OneCounter {}

impl CircuitIO for OneCounter {
    type I = Signal<b8, Red>;
    type O = Signal<b4, Red>;
    type Kernel = one_counter;
}

impl CircuitDQ for OneCounter {
    type D = ();
    type Q = ();
}

#[kernel]
pub fn one_counter(input: Signal<b8, Red>, _q: ()) -> (Signal<b4, Red>, ()) {
    let input = input.val();
    let count = helper::count_ones::<U8, U4>(input);
    (signal(count), ())
}
```

We can now test it using the same method as the before:

```rust,write:ones/tests/test_one_counter_divided.rs
use rhdl::prelude::*;

#[test]
fn test_ones_counter_divided() -> miette::Result<()> {
    let inputs = (0..256).map(b8).map(signal).uniform(100);
    let uut = ones::divided::OneCounterDivided {};
    uut.run(inputs).for_each(|s| {
        let input = s.value.0.val();
        let output_count = s.value.1.val().raw();
        let count_expected = input.raw().count_ones() as u128;
        assert_eq!(output_count, count_expected);
    });
    Ok(())
}
```

```shell,rhdl:ones
cargo nextest run
```

Rerunning the timing calculation is now as simple as making a new test

```rust,write:ones/tests/test_timing_divided.rs
#[test]
fn test_timing_divided() -> miette::Result<()> {
    let uut = ones::divided::OneCounterDivided {};
    let timing = rhdl_toolchains::icestorm::IceStorm::new("hx8k", "cb132", "build").time(uut)?;
    eprintln!("Timing: {timing:?}");
    eprintln!(
        "Total delay: {} nsec",
        timing.logic_delay + timing.routing_delay
    );
    Ok(())
}
```

If we run this test, we get an updated timing result

```shell,rhdl:ones
cargo nextest run test_timing_divided --no-capture
```

We now have a timing of 9.78 nsec, which is significantly faster.  You can flash this design and test it out as an exercise.


