# Hardware Testing

We will repeat the exercise of the `Xor` gate.  This time our fixture will look like this:

```badascii
    +-+Fixture+-------+        
a +-+---+  +-----+    |  
    |   |  | Half+----+-> sum       
    |   +->| Add |    |   
b +-+---+  |     +----+-> carry        
    |      +-----+    |        
    +-----------------+        
```

We will then use a constraints file to bind `a, b, sum, carry` to pins on the FPGA.  Using the `bind!` makes this pretty simple:

```rust,write:half/tests/test_fixture.rs
use rhdl::prelude::*;

#[test]
fn test_make_fixture() -> miette::Result<()> {
    let mut fixture = Fixture::new("half_top", half::HalfAdder::default());
    bind!(fixture, a -> input.val().0);
    bind!(fixture, b -> input.val().1);
    bind!(fixture, sum -> output.sum.val());
    bind!(fixture, carry -> output.carry.val());
    let vlog = fixture.module()?;
    eprintln!("{vlog}");
    std::fs::write("half_top.v", vlog.to_string()).unwrap();
    Ok(())
}
```

```shell,rhdl:half
cargo build -q
cargo test --test test_fixture -- --no-capture
```

We now need a constraints file to map the `a, b, sum, carry` ports of our top level module to pins on the FPGA.  The constraint file is again magic-ed into existence:

```rust,write:half/half.pcf
set_io a H11
set_io b G11
set_io sum E12
set_io carry D14
```

And again, we use the `just` tool to pack these into a simple task:

```rust,write:half/Justfile
build:
    yosys -p 'synth_ice40 -top half_top -json half_top.json' half_top.v
    nextpnr-ice40 --hx8k --json half_top.json --pcf half.pcf --asc half.asc --package cb132
    icepack half.asc half.bin
    openfpgaloader --verify -b ice40_generic half.bin
```


