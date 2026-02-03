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

```rust
{{#rustdoc_include ../code/src/half_adder.rs:adder-step-17}}
```

The generated verilog is verbose, but readable.  RHDL assumes your toolchain will optimize away the fixture logic, which is just wiring.

```verilog
{{#include ../code/half_adder_fixture.v}}
```

At this point, we can now build and flash the FPGA (if we have it).  The following test will do the trick:

```rust
{{#rustdoc_include ../code/src/half_adder.rs:adder-step-18}}
```

At this point your board should be running the `half` circuit!  Congratulations!


