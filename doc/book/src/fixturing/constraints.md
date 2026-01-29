# Constraints

Generally, toolchains will want a constraints file when mapping a design to physical hardware, such as an FPGA.  The constraints file provides information about which physical pins map to which logical inputs on the design, as well as timing constraints and other information used to configure the chip.  In general, these constraint files are highly specific to the toolchain and/or family of FPGA being targetted.  As such, RHDL does not attempt to generate constraint files automatically.  Instead, `Driver` implementations can provide constraint information that is simply concatenated into a constraints file for the entire design.

To see how this works, we will use an example that is specific to the Vivado toolchain, and the XEM7010 FPGA board from Opal Kelly.  This board uses a Xilinx Artix-7 FPGA, and Vivado is the toolchain used to synthesize designs for this FPGA.

Although the details of the circuit being used are not important, here is the top level circuit defintion, which is wraps a 32 bit counter.  First the struct that contains the counter as a child element:

```rust
{{#rustdoc_include ../code/src/fixturing.rs:blinky-U}}
```

Next, we have the `SynchronousIO` implementation for the circuit, which as no inputs, but outputs an 8-bit value to drive the 8 LEDs

```rust
{{#rustdoc_include ../code/src/fixturing.rs:blinky-io}}
```

Finally, we have the kernel definition for the circuit, which simply takes a high bit of the counter, and then toggles the LED pattern based on that bit.  It also ties the enable line high so that the counter is always counting.

```rust
{{#rustdoc_include ../code/src/fixturing.rs:blinky-kernel}}
```

With these pieces in place, we can now build a fixture for this circuit.  

First, we have to use the `Adapter` to convert the synchronous circuit into a standard circuit:

```rust
{{#rustdoc_include ../code/src/fixturing.rs:blinker-adapter}}
```

Next, we create a fixture for this circuit, with a generic name of `top` for the top level module:

```rust
{{#rustdoc_include ../code/src/fixturing.rs:blinker-fixture-start}}
```

Now, we create instances of the input and output values for the circuit using the `dont_care()` method.  Note that these are the inputs and outputs of the adapter-wrapped circuit, not our underlying synchronous design.  The `.io` method on the `Fixture` makes it a one liner:

```rust
{{#rustdoc_include ../code/src/fixturing.rs:blinker-fixture-io}}
```

Finally, we add the 3 drivers of interest.  In this case, we will add a clock driver to connect the circuit to the system clock provided on the board, a constant input driver to tie the reset line low (Xilinx FPGAs generally do not require a reset), and a driver to connect the output of the circuit to the 8 LEDs on the board:

```rust
{{#rustdoc_include ../code/src/fixturing.rs:blinker-fixture-drivers}}
```

The total fixture code looks like this:

```rust
{{#rustdoc_include ../code/src/fixturing.rs:blinker-fixture}}
```

The resulting Verilog looks like this:

```verilog
{{#include ../code/blinky_fixture.v}}
```

And the constraint file looks like this:

```tcl
{{#include ../code/blinky_constraints.xdc}}
```
