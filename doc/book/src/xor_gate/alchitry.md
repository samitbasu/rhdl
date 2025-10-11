# Alchitry FPGA Cu v2

This section describes how to take a simple RHDL circuit, such as the XorGate, and deploy it onto a physical device. It covers the process of creating a fixture, connecting drivers, exporting to Verilog, and synthesizing the design for an FPGA. Each step is explained in detail in the following subsections.

I will assume you have an `Alchitry Cu v2` FPGA board and the `Alchitry Io v2` interface board.  But you could easily adapt this to other boards.  I just need something concrete to illustrate the process.  You will also need the `icestorm` toolchain.  For Mac OS X, you can install this with:

```shell
 ❯ brew tap samitbasu/oss-fpga
 ❯ brew install --HEAD icestorm yosys nextpnr-ice40 openfpgaloader
```

For other platforms, follow the various build steps or use your built-in package manager.
