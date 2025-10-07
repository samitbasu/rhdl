## Making Trace Files

There are other things you can do with an output iterator from the `.run` method.  One neat thing you can do in RHDL is to generate a VCD file or even a SVG of a trace display.  The `Vcd` container can collect the output of the simulation, and then be written to either type of file.  Using the `Vcd` container is extremely simple, you just `.collect` the iterator into it.  Consult the documentation to see what options the `svg` export supports.  You can filter traces, and adjust the sizes of various elements in the rendered image.

Here is the updated test

```rust,write:xor/tests/test_svg.rs
use rhdl::prelude::*;

#[test]
fn test_svg() -> miette::Result<()> {
    let inputs = [(false, false), (false, true), (true, false), (true, true)];
    let it = inputs.into_iter().cycle().take(5).map(signal).uniform(100);
    let uut = xor::XorGate;
    let vcd: Vcd = uut.run(it).collect();
    let svg = vcd.dump_svg(&SvgOptions::default());
    std::fs::write("xor.svg", svg.to_string()).unwrap();
    Ok(())
}
```

```shell,rhdl:xor
cargo build -q
cargo test --test test_svg
```

The resulting SVG shows the input and output signals as one would expect for a trace file.

```shell,rhdl-silent:xor
cp xor.svg $ROOT_DIR/src/img/.
```
![XorGate Simulation](../img/xor.svg)

You can also generate a traditional `VCD` file which can be opened by other tools like [surfer](https://surfer-project.org/).  Here is a test file to generate a `.vcd` file.

```rust,write:xor/tests/test_vcd.rs
use rhdl::prelude::*;

#[test]
fn test_vcd() -> miette::Result<()> {
    let inputs = [(false, false), (false, true), (true, false), (true, true)];
    let it = inputs.into_iter().cycle().take(5).map(signal).uniform(100);
    let uut = xor::XorGate;
    let vcd: Vcd = uut.run(it).collect();
    let file = std::fs::File::create("xor.vcd").unwrap();
    vcd.dump(file).unwrap();
    Ok(())
}
```

```shell,rhdl:xor
cargo build -q
cargo test --test test_vcd
```

Here is a screen shot of the VCD as rendered by `surfer`:

![Surfer Screenshot](../img/surfer_xor_2.png)
