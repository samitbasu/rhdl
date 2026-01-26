# Testbenches

RHDL allows you to collect a trace and build an testbench from it.  This can be handy when you want to validate your design using a different simulation tool, or check that the RHDL synthesized HDL matches the expected behavior.  There are two different testbench containers provided, depending on if the circuit being simulated is `Circuit` or `Synchronous`.  

In general, the process involves 3 steps:

- Generate a simulation trace that exercises the circuit behavior you want to test.  Make sure to collect all trace samples, since dropping trace samples will lead to an incomplete or nonfunctional testbench.
- Collect the simulation trace into a `TestBench` container (for `Circuit` designs) or a `SynchronousTestBench` container (for `Synchronous` designs).
- Generate either an RTL or NetList testbench from the testbench container.  You can then write out the testbench and run it through your simulator of choice.

The three steps are illustrated in the following example, which uses a simple AND gate

```rust
{{#rustdoc_include ../code/src/testbench.rs:AND_Testbench}}
```

The resulting testbench file looks like this:

```verilog
{{#include ../code/and_test_tb.v}}
```

It includes both the stimulus sequence for the circuit, as well as the expected output values for each input combination.  It also includes RTL description of the circuit itself.  If you run the testbench, you should get a `TESTBENCH OK` message at the end, indicating that the circuit output matched the expected values.

<!-- cmdrun to-html "cd ../code && iverilog and_test_tb.v && ./a.out" -->

At the moment, the NetList export from a testbench is not recommended for general use.  

The process with synchronous circuits is analogous.  Simply use a `SynchronousTestBench` container instead of a `TestBench` container.

```rust
{{#rustdoc_include ../code/src/testbench.rs:counter_testbench}}
```

The resulting testbench file looks like this:

```verilog
{{#include ../code/counter_test_tb.v}}
```

It includes clock and reset generation, as well as the stimulus sequence and expected output values.  Running the testbench should also yield a `TESTBENCH OK` message at the end.

<!-- cmdrun to-html "cd ../code && iverilog counter_test_tb.v && ./a.out" -->
