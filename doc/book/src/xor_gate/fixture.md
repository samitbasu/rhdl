### Fixture

The idea of a `Fixture` is meant to convey the notion of an external support that holds your circuit and provides the inputs and outputs that it needs to communicate with the outside world.  That outside world might be another set of Verilog modules, a physical device, or some other environment.  Ultimately, there are code and config pieces that need to be provided for the circuit you designed to get inputs from the physical world and provide outputs to feed them back.

The concept looks something like this:
```badascii
+---------+Fixture+------------------------------+
|  pin +------+                    +------+pin   |
|  +-->|Driver+-+               +->|Driver+--->  |
|I     +------+ |               |  +------+     O|
|N              |               |               U|
|P pin +------+ | I +-------+ O |  +------+pin  T|
|U +-->|Driver+-+-->|Circuit+---+->|Driver+---> P|
|T     +------+ |   +-------+   |  +------+     U|
|S              |               |               T|
|  pin +------+ |               |  +------+pin  S|
|  +-->|Driver+-+               +->|Driver+--->  |
|      +------+                    +------+      |
+------------------------------------------------+
```

A `Driver` is a piece of code and configuration that feeds signals from a physical port or pin to the circuit.  It may also provide a path for the circuit output to a physical port or pin.  Drivers can be more complicated and provide both input and output capabilities.  For now, we will just need basic drivers.  Basic input/output drivers can be created with the `bind!` macro.  For our `XorGate`, we will create something that looks like this:

```badascii
    +-+Fixture+-------+      
    |                 |      
a +-+---+    +-----+  |      
    |   |    | XoR |  |      
    |   +--->| Gate+--+-> y
b +-+---+    |     |  |      
    |        +-----+  |      
    +-----------------+      
```

We will then use a constraints file to bind `a, b, y` to pins on the FPGA.  Using the `bind!` macro this is pretty simple:

```rust
{{#rustdoc_include ../code/src/xor.rs:xor-step-18}}
```

The `Fixture` provides a convenient `.io_dont_care()` method that returns a tuple of the input and output types of the circuit, with `dont_care` values for the fields.  This provides a convenient way to get an instance of the `input` and `output` types that are needed by the `bind!` macro.  


The generated Verilog for the fixture looks like this:

```verilog
{{#include ../code/xor_top.v}}
```

Note that the top level module is called `xor_top` as we specified when we created the Fixture.  The pins `a, b, y` are all present as top level ports.  The `bind!` macro created the necessary wiring to connect the pins to the internal `XorGate` instance.
