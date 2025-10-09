# Testing the Half Adder

Let's test our half adder.  We will skip straight to the iterator based testing methods, since direct testing with explicit calls to `sim` are tedious.  We can again benefit from the fact that the test code is just Rust, and can thus easily compute the expected values using whatever functions we like, without regard for worrying about their synthesizability.


```shell,rhdl:half
mkdir tests
```

```rust,write:half/tests/test_half_adder.rs
use rhdl::prelude::*;

#[test]
fn test_half_adder() -> miette::Result<()> {
    let inputs = [(false, false), (false, true), (true, false), (true, true)];
    let it = inputs.into_iter().cycle().take(5).map(signal).uniform(100);
    let uut = half::HalfAdder::default();
    uut.run(it).for_each(|s| {
        let input = s.value.0.val();
        let output_sum = s.value.1.sum.val();
        let output_carry = s.value.1.carry.val();
        let sum_expected = input.0 ^ input.1;
        let carry_expected = input.0 & input.1;
        assert_eq!(output_sum, sum_expected);
        assert_eq!(output_carry, carry_expected);
    });
    Ok(())
}
```

As a reminder, the `run` method on any `Circuit` produces a sequence of `TimedSample<(I,O)>`, which has
 
- `s.time` the timestamp of the sample (monotonic, but not strictly)
- `s.value` a tuple containing the input and corresponding output for that time sample
- In this example, `s.value` is of type `TimedSample<(Signal<(bool, bool), Red>, Outputs)>`

```shell,rhdl:half
cargo build -q
cargo test --test test_half_adder
```

Let us also make a Verilog testbench that validates our translation of the code for synthesis.

```rust,write:half/tests/test_tb_rtl.rs
use rhdl::prelude::*;

#[test]
fn test_testbench() -> miette::Result<()> {
    let inputs = [(false, false), (false, true), (true, false), (true, true)];
    let it = inputs.into_iter().cycle().take(5).map(signal).uniform(100);
    let uut = half::HalfAdder::default();
    let tb: TestBench<_, _> = uut.run(it).collect();
    let tb = tb.rtl(&uut, &TestBenchOptions::default())?;
    std::fs::write("half_rtl_tb.v", tb.to_string()).unwrap();
    Ok(())
}
```

We can generate the testbench file by running the test:

```shell,rhdl:half
cargo build -q
cargo test --test test_tb_rtl
cat half_rtl_tb.v
```

We can again test it using `icarus`

```shell,rhdl:half
iverilog half_rtl_tb.v
./a.out
```

We can also generate a netlist representation and test that

```rust,write:half/tests/test_tb_ntl.rs
use rhdl::prelude::*;

#[test]
fn test_testbench_ntl() -> miette::Result<()> {
    let inputs = [(false, false), (false, true), (true, false), (true, true)];
    let it = inputs.into_iter().cycle().take(5).map(signal).uniform(100);
    let uut = half::HalfAdder::default();
    let tb: TestBench<_, _> = uut.run(it).collect();
    let tb = tb.ntl(&uut, &TestBenchOptions::default())?;
    std::fs::write("half_tb_ntl.v", tb.to_string()).unwrap();
    Ok(())
}
```
We can generate the testbench file again, and run it through `icarus`:

```shell,rhdl:half
cargo build -q
cargo test --test test_tb_ntl
cat half_tb_ntl.v
```

```shell,rhdl:half
iverilog half_tb_ntl.v
./a.out
```

Finally, we can generate a trace file to see the test cases as a timeseries

```rust,write:half/tests/test_svg.rs
use rhdl::prelude::*;

#[test]
fn test_svg() -> miette::Result<()> {
    let inputs = [(false, false), (false, true), (true, false), (true, true)];
    let it = inputs.into_iter().cycle().take(5).map(signal).uniform(100);
    let uut = half::HalfAdder::default();
    let vcd: Vcd = uut.run(it).collect();
    let svg = vcd.dump_svg(&SvgOptions::default());
    std::fs::write("half.svg", svg.to_string()).unwrap();
    Ok(())
}
```

```shell,rhdl:half
cargo build -q
cargo test --test test_svg
```

The resulting SVG shows the input and output signals as one would expect for a trace file.

```shell,rhdl-silent:half
cp half.svg $ROOT_DIR/src/img/.
```

![HalfAdder Simulation](../../img/half.svg)
