## Verilog Testbenches

The last way to test your circuit is to generate a Verilog testbench and run the entire thing through a standalone simulation tool.  This can also be done using the iterator produced by the `.run` method.  But instead of collecting into a `Vcd`, we will collected into a `TestBench` object.  Once we have a `TestBench`, we can generate a test module at either the RTL (Register Transfer Language) level or the NTL (Netlist) level.  Which one you want to use depends on the nature of your test.  The RTL description is closer to the top level Rust code, and easier to map from the source code to the corresponding Verilog entity.  But the NTL description is also useful for more low level testing.  

Let's start with the simplest version, we collect the output into a testbench, and then write it to Verilog with a RTL description of the circuit:

```rust,write:xor/tests/test_tb_rtl.rs
use rhdl::prelude::*;

#[test]
fn test_testbench() -> miette::Result<()> {
    let inputs = [(false, false), (false, true), (true, false), (true, true)];
    let it = inputs.into_iter().cycle().take(5).map(signal).uniform(100);
    let uut = xor::XorGate;
    let tb: TestBench<_, _> = uut.run(it).collect();
    let tb = tb.rtl(&uut, &TestBenchOptions::default())?;
    std::fs::write("xor_tb.v", tb.to_string()).unwrap();
    Ok(())
}
```

We can generate the testbench file by running the test:

```shell,rhdl:xor
cargo build -q
cargo test --test test_tb_rtl
cat xor_tb.v
```

The simplest thing we can do at this point is run the testbench.  Assuming you have a tool like `icarus` installed, you can execute the testbench as:

```shell,rhdl:xor
iverilog xor_tb.v
./a.out
```

If we wanted a lower level (netlist) representation, we can use the `.ntl` method on the `TestBench`.  Generally, netlist manipulation is not a core element of RHDL - it is used as an analysis tool to check for various design issues.  But you can still check that the output simulates correctly:

```rust,write:xor/tests/test_tb_ntl.rs
use rhdl::prelude::*;

#[test]
fn test_testbench_ntl() -> miette::Result<()> {
    let inputs = [(false, false), (false, true), (true, false), (true, true)];
    let it = inputs.into_iter().cycle().take(5).map(signal).uniform(100);
    let uut = xor::XorGate;
    let tb: TestBench<_, _> = uut.run(it).collect();
    let tb = tb.ntl(&uut, &TestBenchOptions::default())?;
    std::fs::write("xor_tb_ntl.v", tb.to_string()).unwrap();
    Ok(())
}
```
We can generate the testbench file again, and run it through `icarus`:

```shell,rhdl:xor
cargo build -q
cargo test --test test_tb_ntl
cat xor_tb_ntl.v
```

```shell,rhdl:xor
iverilog xor_tb_ntl.v
./a.out
```
