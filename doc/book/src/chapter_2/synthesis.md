### Synthesis

So far, we have remained in relatively device-agnostic territory.  But now we need to adopt some tooling to get something we can put on the FPGA of interest.  For the `Io` board, we want to use the first two dip switches of the leftmost switch bank to provide the `a` and `b` inputs to our XorGate. Tracing the schematic through the pins is a bit tricky (there is a left-right reversal as you go between the IO board and the base FPGA board).  To get moving, we will just hard code the pin constraint file (PCF) using the following 

```rust,write:xor/xor.pcf
set_io a H11
set_io b G11
set_io y E12
```

Note that the PCF file is case sensitive.  Next, we run the synthesis using `yosys`, and the place and route using `nextpnr-ice40`.  To simplify, I will use the `just` tool to pack these into a simple task:

```rust,write:xor/Justfile
build:
    yosys -p 'synth_ice40 -top xor_top -json xor_top.json' xor_top.v
    nextpnr-ice40 --hx8k --json xor_top.json --pcf xor.pcf --asc xor.asc --package cb132
    icepack xor.asc xor.bin
    # iceprog xor.bin - iceprog does not correctly flash the Cu v2
    openfpgaloader --verify -b ice40_generic xor.bin
```
