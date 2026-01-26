# Roll Your Own

While RHDL provides a number of convenience methods and extension traits to enable iterator-based simulations, you don't have to use them.  You can always roll your own simulation loop if you want to.  The ease with which you can simulate your design can be important when working with advanced features, like state-precomputation.  


Here is a simple example of a simulation loop that uses none of the iterator or trace infrastructure provided by RHDL.  The `XorGate` circuit is defined [here](../xor_gate/the_gate.md).  The simulation loop is as follows:

```rust
{{#rustdoc_include ../../code/src/simulations.rs:roll-sim}}
```

The key steps are:

1. Create an instance of the circuit to be simulated.
2. Initialize the circuit state.
3. For each input sample:
   - Set the circuit input.
   - Call the `sim` method to advance the circuit state and compute the output.
   - Collect or process the output as desired.

It's quite simple.  There is no magic or mystery here.  One particular use case for a hand-rolled simulation setup is when you want to connect your circuit to outside stimulus.  For example, you may want to process bytes from a network or a file.  Or you may want to feed the output of your circuit simulator to some other system.  In these cases, you can simply call the `.sim` method on your circuit as needed with new inputs, and process the outputs as they are produced.
