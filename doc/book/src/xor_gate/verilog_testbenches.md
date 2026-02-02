## Verilog Testbenches

The last way to test your circuit is to generate a Verilog testbench and run the entire thing through a standalone simulation tool.  This can also be done using the iterator produced by the `.run` method.  But instead of collecting into a `VcdFile`, we will collected into a `TestBench` container.  Once we have a `TestBench`, we can generate a test module at either the RTL (Register Transfer Language) level or the NTL (Netlist) level.  Which one you want to use depends on the nature of your test.  The RTL description is closer to the top level Rust code, and easier to map from the source code to the corresponding Verilog entity.  But the NTL description is also useful for more low level testing.  

Let's start with the simplest version, we collect the output into a testbench, and then write it to Verilog with a RTL description of the circuit:

```rust
{{#rustdoc_include ../code/src/xor.rs:xor-step-16}}
```
Running this produces a Verilog testbench file `xor_tb.v`:

```verilog
{{#include ../code/xor_tb.v}}
```

The simplest thing we can do at this point is run the testbench.  Assuming you have a tool like `icarus` installed, you can execute the testbench as:

<!-- cmdrun to-html "cd ../code && iverilog xor_tb.v && ./a.out" -->

If we wanted a lower level (netlist) representation, we can use the `.ntl` method on the `TestBench`.  Generally, netlist manipulation is not a core element of RHDL - it is used as an analysis tool to check for various design issues.  But you can still check that the output simulates correctly:

```rust
{{#rustdoc_include ../code/src/xor.rs:xor-step-17}}
```

The updated testbench has the the netlist description of the gate (which is trivial in this case):

```verilog
{{#include ../code/xor_tb_ntl.v}}
```

<!-- cmdrun to-html "cd ../code && iverilog xor_tb_ntl.v && ./a.out" -->
