# Hardware Testing

We need to fixture our ones counter so that it looks like this:

```badascii
             +-----+Fixture+----+           
 dip0..dip7  |  ++OnesCounter++ | led0..led3
+------------+->|             +-+---->      
             |  +-------------+ |           
             +------------------+           
```

We will then use the constraints file to bind the `dip` switches and `led` pins to the FPGA.  Again, we use the `bind!` macro to connect top level named ports to the inputs and outputs of our circuit:

```rust,write:ones/tests/test_fixture.rs
use rhdl::prelude::*;

#[test]
fn test_make_fixture() -> miette::Result<()> {
    let mut fixture = Fixture::new("ones_top", ones::OneCounter {});
    bind!(fixture, dips -> input.val());
    bind!(fixture, leds -> output.val());
    let vlog = fixture.module()?;
    eprintln!("{vlog}");
    Ok(())
}
```

```shell,rhdl:ones
cargo build -q
cargo test --test test_fixture -- --no-capture
```

Let's now create an integration test to build and flash the FPGA

```rust,write:ones/tests/test_flash.rs
use rhdl::prelude::*;

#[test]
#[ignore]
fn test_build_flash() -> miette::Result<()> {
    const PCF: &str = "
set_io dips[0] H11
set_io dips[1] G11
set_io dips[2] F11
set_io dips[3] E11
set_io dips[4] D11
set_io dips[5] D10
set_io dips[6] G1
set_io dips[7] D9
set_io leds[0] E12
set_io leds[1] D14
set_io leds[2] F12
set_io leds[3] E14
    ";
    let mut fixture = Fixture::new("ones_top", ones::OneCounter {});
    bind!(fixture, dips -> input.val());
    bind!(fixture, leds -> output.val());
    rhdl_toolchains::icestorm::IceStorm::new("hx8k", "cb132", "build")
        .clean()?
        .build_and_flash(fixture, PCF)
}
```

If you plug in your board, and run the test, you should have a functioning ones counter!

```shell,rhdl:ones
cargo nextest run test_build_flash -- --ignored
```
