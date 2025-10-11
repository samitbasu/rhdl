# Testing the One Counter

Let's test our one-counter.  We will again use iterator-based testing, since this circuit is simple enough to use.  

```shell,rhdl:ones
mkdir tests
```

```rust,write:ones/tests/test_ones_counter.rs
use rhdl::prelude::*;

#[test]
fn test_ones_counter() -> miette::Result<()> {
    let inputs = (0..256).map(b8).map(signal).uniform(100);
    let uut = ones::OneCounter {};
    uut.run(inputs).for_each(|s| {
        let input = s.value.0.val();
        let output_count = s.value.1.val().raw();
        let count_expected = input.raw().count_ones() as u128;
        assert_eq!(output_count, count_expected);
    });
    Ok(())
}
```

Here, we use the Rust `count_ones()` method to get the expected output, and thus trivially check that our naive implementation gets the right answer.  Note that you can call `.raw()` on a `Bits` type to get the underlying `u128` representation, but not in synthesizable code (i.e., not in a kernel).  

```shell,rhdl:ones
cargo test
```

We can also make a Verilog testbench that validates our translation of the code for synthesis.  Here, it is handy that `rustc` computed the expected values for us - the test is now checking the Verilog against the Rust.  Correctness of the Rust code was already established above.

```rust,write:ones/tests/test_tb_rtl.rs
use rhdl::prelude::*;

#[test]
fn test_testbench() -> miette::Result<()> {
    let inputs = (0..256).map(b8).map(signal).uniform(100);
    let uut = ones::OneCounter {};
    let tb: TestBench<_, _> = uut.run(inputs).collect();
    let tb = tb.rtl(&uut, &TestBenchOptions::default())?;
    std::fs::write("ones_rtl_tb.v", tb.to_string()).unwrap();
    Ok(())
}
```

We can generate the testbench file by running the test:

```shell,rhdl:ones
cargo build -q
cargo test --test test_tb_rtl
tail -50 ones_rtl_tb.v
```

We can also test it using `icarus`

```shell,rhdl:ones
iverilog ones_rtl_tb.v
./a.out
```

We can also generate the netlist representation and test that


```rust,write:ones/tests/test_tb_ntl.rs
use rhdl::prelude::*;
use rhdl::prelude::*;

#[test]
fn test_testbench_ntl() -> miette::Result<()> {
    let inputs = (0..256).map(b8).map(signal).uniform(100);
    let uut = ones::OneCounter {};
    let tb: TestBench<_, _> = uut.run(inputs).collect();
    let tb = tb.ntl(&uut, &TestBenchOptions::default())?;
    std::fs::write("ones_tb_ntl.v", tb.to_string()).unwrap();
    Ok(())
}
```

We can generate the testbench file again, and run it through `icarus`:

```shell,rhdl:ones
cargo build -q
cargo test --test test_tb_ntl
tail -50 ones_tb_ntl.v
```

If you look at the output of the netlist, it looks slightly cleaner than the output of the RTL.  You can think of the RTL as only lightly optimized, and the netlist as more heavily optimized.  Those optimizations can yield much leaner structures, but also make it harder to trace back to the original Rust source.

And running it through `icarus`:

```shell,rhdl:ones
iverilog ones_tb_ntl.v
./a.out
```

Finally, we can generate a trace file to see the test cases as a timeseries.  A useful technique demonstrated here is filtering out the timestamps to only keep a small section of the trace.  The filtering can be arbitrary.  So if you only want to keep timestamps once a certain condition is met, you can easily do so...  

```rust,write:ones/tests/test_svg.rs
use rhdl::prelude::*;

#[test]
fn test_svg() -> miette::Result<()> {
    let inputs = (0..256).map(b8).cycle().take(257).map(signal).uniform(100);
    let uut = ones::OneCounter {};
    let vcd: Vcd = uut.run(inputs).skip_while(|t| t.time < 25000).collect();
    let svg = vcd.dump_svg(&SvgOptions::default());
    std::fs::write("ones.svg", svg.to_string()).unwrap();
    Ok(())
}
```

```shell,rhdl:ones
cargo build -q
cargo test --test test_svg
```

The resulting SVG shows the input and output signals as one would expect for a trace file.

```shell,rhdl-silent:ones
cp ones.svg $ROOT_DIR/src/img/.
```

![OnesCounter Simulation](../../img/ones.svg)


