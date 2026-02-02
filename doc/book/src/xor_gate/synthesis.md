### Synthesis

So far, we have remained in relatively device-agnostic territory.  But now we need to adopt some tooling to get something we can put on the FPGA of interest.  For the `Io` board, we want to use the first two dip switches of the leftmost switch bank to provide the `a` and `b` inputs to our XorGate. Tracing the schematic through the pins is a bit tricky (there is a left-right reversal as you go between the IO board and the base FPGA board). 

You will also need to drive the toolchain for your board, and provide it the options, and parameters it needs to understand your particular FPGA board.  This is generally quite difficult - there is no single solution that will work for multiple boards.  Part of the value of FPGAs is that they are adaptable and configurable, and understanding exactly what FPGA you have, and how it is configured is detailed knowledge that RHDL cannot possibly know in advance.

That being said, there are only a handful of toolchains in the wild, and there are helper functions for these in the `rhdl_toolchains` crate.  In general, these are shells out to command line tools or script-generators.  There are no real great options for avoiding shelling out at the moment.

```shell
cargo add --dev rhdl-toolchains
```

When mapping a design to an FPGA, you will also need a constraint file to tell the toolchain which signals in your circuit map to which pins on the physical device.  For the Alchitry Cu, we will be using the `icestorm` toolchain, which requires a "pin constraint file" (PCF).  

I like to put the build and flash logic into an integration test that I can run when I have the FPGA board attached to the host.  As such, I normally mark them as `#[ignore]` so they only run when we want them to run.  

```rust
{{#rustdoc_include ../code/src/xor.rs:xor-step-19}}
```

There are a lot of FPGA specific details here.  First, note that the PCF file maps the inputs of the generated module to specific pins (or balls) of the FPGA.  Next, the `IceStorm` constructor wants to know the part kind and package for our FPGA.  These are specific to the Alchitry Cu board.  Finally, we provide a build directory for the process to use.

This test will only work if the FPGA board is attached to your host machine.  At this point your board should be running the `xor` circuit!  Congratulations!
