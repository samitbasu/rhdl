# Fixturing and Synthesis

The idea of a `Fixture` is meant to convey the notion of an external support that holds your circuit and provides the inputs and outputs that it needs to communicate with the outside world.  That outside world might be another set of Verilog modules, a physical device, or some other environment.  Ultimately, there are code and config pieces that need to be provided for the circuit you designed to get inputs from the physical world and provide outputs to feed them back.

A fixture is a top level module that contains a circuit, and allows you to attach various drivers to drive inputs and observe outputs.  A fixture is helpful when you  need to take a RHDL `Circuit` and connect it to some external system (like
a testbench or a hardware description language).

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

A `Driver` is a piece of (Verilog) code and configuration that feeds signals from a physical port or pin to the circuit.  It may also provide a path for the circuit output to a physical port or pin.  Drivers can be more complicated and provide both input and output capabilities.  

For example, when using a hard IP core on an FPGA, you may have a single driver that instantiates the hard IP core, and then connects it to the circuit.

```badascii
+-+Fixture+------------------------------+   
|        inputs        ++DRAM++          |   
|+---------------+     |Driver|  Out pins|   
||               |<----+      +----------+-->
||  +-------+    |<----+      |Input pins|   
|+->|Circuit|    +     |      |<---------+--+
|   |       |     +--->|      |Inout pins|   
|   |       +---->|    |      |<---------+-->
|   |       |     +--->|      |          |   
|   +-------+  outputs +------+          |   
+----------------------------------------+   
```

Drivers connect to the input circuit via `MountPoint`s.  A mount point
is simply a range of bits on the input or output of the circuit.  It is best illustrated with a diagram:

```badascii
                     Input            
pin x+------+        +-+   Mount Point
+--->|      |bits<2> |8|   Input(7..9)
pin y|Driver+------->|7|<------+      
+--->|      |        +-+              
     +------+        | |              
                     |.|              
         Other +---->|.|              
       Drivers       |.|              
                     | |              
                     +-+              
```

Finally, the `bind` macro is a helper to make it easier to connect
inputs and outputs of a circuit to the top level of the fixture.
It allows you to specify paths on the input and output of the circuit,
and automatically creates the necessary drivers and mount points.

