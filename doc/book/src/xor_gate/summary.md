# Chapter 2 - A Simple Xor Gate

We will walk through the process of building and testing a simple Xor gate.  This is the simplest possible asynchronous circuit, and contains no internal sub-components.  But in the process, we can learn much about RHDL, including how to test and synthesize the resulting circuits.

## Key Concepts
The key concepts of this chapter:

- The `Circuit` trait
- The `I` and `O` types for inputs and outputs
- The `Kernel` to describe interconnect and behavior
- Testing
- Iterator based testing
- Making trace files
- Fixtures
- Synthesis
- Toolchains
- Hardware demo!

It's a lot, but each step is reasonably small.  Let's go!
