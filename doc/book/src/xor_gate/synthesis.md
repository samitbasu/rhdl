### Synthesis

So far, we have remained in relatively device-agnostic territory.  But now we need to adopt some tooling to get something we can put on the FPGA of interest.  For the `Io` board, we want to use the first two dip switches of the leftmost switch bank to provide the `a` and `b` inputs to our XorGate. Tracing the schematic through the pins is a bit tricky (there is a left-right reversal as you go between the IO board and the base FPGA board). 

You will also need to drive the toolchain for your board, and provide it the options, and parameters it needs to understand your particular FPGA board.  This is generally quite difficult - there is no single solution that will work for multiple boards.  Part of the value of FPGAs is that they are adaptable and configurable, and understanding exactly what FPGA you have, and how it is configured is detailed knowledge that RHDL cannot possibly know in advance.

That being said, there are only a handful of toolchains in the wild, and there are helper functions for these in the `rhdl_toolchains` crate.  In general, these are shells out to command line tools or script-generators.  There are no real great options for avoiding shelling out at the moment.

```shell,rhdl:xor
cargo add --path ~samitbasu/Devel/rhdl/crates/rhdl-toolchains
```

When mapping a design to an FPGA, you will also need a constraint file to tell the toolchain which signals in your circuit map to which pins on the physical device.  For the Alchitry Cu, we will be using the `icestorm` toolchain, which requires a "pin constraint file" (PCF).  

I like to put the build and flash logic into an integration test that I can run when I have the FPGA board attached to the host.  As such, I normally mark them as `#[ignore]` so they only run when we want them to run.  

Let's test that everything looks good first:

```shell,rhdl:xor
# Install with cargo install --locked cargo-nextest
cargo nextest run
```

Finally, we can create an integration test to build and flash the FPGA

```rust,write:xor/tests/test_flash.rs
use rhdl::prelude::*;

#[test]
#[ignore]
fn test_build_flash() -> miette::Result<()> {
    const PCF: &str = "
set_io a H11
set_io b G11
set_io y E12    
    ";
    let uut = xor::XorGate;
    let mut fixture = Fixture::new("xor_flash", uut);
    bind!(fixture, a -> input.val().0);
    bind!(fixture, b -> input.val().1);
    bind!(fixture, y -> output.val());
    rhdl_toolchains::icestorm::IceStorm::new("hx8k", "cb132", "build").
    clean()?.
    build_and_flash(fixture, PCF)
}
```

There are a lot of FPGA specific details here.  First, note that the PCF file maps the inputs of the generated module to specific pins (or balls) of the FPGA.  Next, the `IceStorm` constructor wants to know the part kind and package for our FPGA.  These are specific to the Alchitry Cu board.  Finally, we provide a build directory for the process to use.

```shell,rhdl:xor
cargo nextest run test_build_flash -- --ignored
```

At this point your board should be running the `xor` circuit!  Congratulations!
