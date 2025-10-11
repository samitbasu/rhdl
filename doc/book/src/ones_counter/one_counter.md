# One Counter

The implementation of the Ones Counter has a few interesting features on its own.

```shell,rhdl-silent
rm -rf ones
```

We repeat the setup steps we had previously:

```shell,rhdl
cargo new --lib ones
cd ones
cargo add --path ~samitbasu/Devel/rhdl/crates/rhdl 
cargo add --path ~samitbasu/Devel/rhdl/crates/rhdl-toolchains
cargo add --dev miette
```

In the `src` directory, we write the following:

```rust,write:ones/src/lib.rs
use rhdl::prelude::*;

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

pub fn one_counter(input: Signal<b8, Red>, _q: ()) -> (Signal<b4, Red>, ()) {
    todo!()
}
```

For the kernel itself, we need to actually count the ones in `b8`.  While the `Bits` type doesn't have a built in `count_ones` method (which is not currently synthesizable), you can easily just count the ones in a `for` loop.  For loops are allowed in synthesizable code under fairly constrained circumstances.  RHDL must be able to unroll the loop at compile time, so the indices must be computable at compile time.  This generally means either literals or simple expressions of `const` parameters.   

Also, we can use a mutable local variable to hold the count.  This is totally fine, as mutable local variables are supported by RHDL.  The end result is a kernel function that looks like this:

```rust
pub fn one_counter(input: Signal<b8, Red>, _q: ()) -> (Signal<b4, Red>, ()) {
    let mut count = b4(0);
    let input = input.val();
    for i in 0..8 {
        if input & (1 << i) != 0 {
            count = count + 1;
        }
    }
    (signal(count), ())
}
```

You can see the logic pretty clearly (which is the point!).  We start with a nibble counter set to zero, and then test the bits one at a time.  This is a "naive" implementation, as it creates a really long chain of adders, but for now, let's leave that alone.  We can improve it if/when needed.  The completed `lib.rs` looks like this:

```rust,write:ones/src/lib.rs
use rhdl::prelude::*;

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
    let mut count = b4(0);
    let input = input.val();
    for i in 0..8 {
        if input & (1 << i) != 0 {
            count = count + 1;
        }
    }
    (signal(count), ())
}
```

```shell,rhdl-silent:ones
cargo check -q
```

```shell,rhdl:ones
cargo check
```

Let's move to testing.