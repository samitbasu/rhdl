# Hardware Testing

We will repeat the exercise of the `Xor` gate.  This time our fixture will look like this:

```badascii
    +-+Fixture+-------+        
a +-+---+  +-----+    |  
    |   |  | Half+----+-> sum       
    |   +->| Add |    |   
b +-+---+  |     +----+-> carry        
    |      +-----+    |        
    +-----------------+        
```

We will then use a constraints file to bind `a, b, sum, carry` to pins on the FPGA.  Using the `bind!` makes this pretty simple:

```rust,write:half/tests/test_fixture.rs
use rhdl::prelude::*;

#[test]
fn test_make_fixture() -> miette::Result<()> {
    let mut fixture = Fixture::new("half_top", half::HalfAdder::default());
    bind!(fixture, a -> input.val().0);
    bind!(fixture, b -> input.val().1);
    bind!(fixture, sum -> output.sum.val());
    bind!(fixture, carry -> output.carry.val());
    let vlog = fixture.module()?;
    eprintln!("{vlog}");
    Ok(())
}
```

```shell,rhdl:half
cargo build -q
cargo test --test test_fixture -- --no-capture
```

At this point, we can now build and flash the FPGA (if we have it).  The following integration test will do the trick:

```rust,write:half/tests/test_flash.rs
use rhdl::prelude::*;

#[test]
#[ignore]
fn test_build_flash() -> miette::Result<()> {
    const PCF: &str = "
set_io a H11
set_io b G11
set_io sum E12
set_io carry D14
    ";
    let mut fixture = Fixture::new("half_top", half::HalfAdder::default());
    bind!(fixture, a -> input.val().0);
    bind!(fixture, b -> input.val().1);
    bind!(fixture, sum -> output.sum.val());
    bind!(fixture, carry -> output.carry.val());
    rhdl_toolchains::icestorm::IceStorm::new("hx8k", "cb132", "build")
        .clean()?
        .build_and_flash(fixture, PCF)
}
```

```shell,rhdl:half
cargo nextest run test_build_flash -- --ignored
```

At this point your board should be running the `half` circuit!  Congratulations!


