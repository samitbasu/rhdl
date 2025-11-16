# Simulations

Simulations are critical to verifying the correctness of the design of your circuits, and RHDL tries really really hard to make sure that you can simulate easily and idiomatically using regular Rust patterns like iterators, collections, assertions, and the like.  The goal is to make the testing as close as possible to the original Rust code that describes the design.  Each additional step of translation or each additional toolchain that gets involved makes it progressively more difficult to understand and localize where the problem in the original design was to begin with.  I can't possibly cover everything about testing in this document, but I'll try to cover some of the more important points and explain how RHDL helps you accomplish them (or not).

- Open loop vs. closed loop testing, i.e., does your test "react" to the output of the circuit under test?  And if so, the requirements for testing are different than if it just "plays" a sequence of inputs out and measures the outputs.
- Exhaustive testing - the pure functional style of the compute kernels suggests that at least in some cases you can check for correctness by exhausting all possible inputs and verifying the outputs.
- Roll your own - Sometimes, the built in simulator loops are just too limited to accomplish what you want.  Fortunately, there is no magic and you can literally write your own simulation loop and drive the circuit.  It is quite simple to do.
- Iterators for simulation - For open loop simulations, iterators make for clean composable tests and readable test cases.
- Probes - these are ways of extracting signals and values from a simulation stream (think `.map` or `.filter`)
- Tracing - Sometimes you need graphical visualization of a simulation result.  RHDL has a pretty good tracing mechanism built in which allows you to log values from within your designs and visualize the output using standard tools.
- Testbenches - RHDL can play some neat tricks, including writing testbenches that check the behavior of generated HDL (e.g., Verilog) against the Rust code.  

There is a lot to cover, and doing so in the abstract is a bit difficult.  So we will use some examples to illustrate along the way.  More example-driven material is presented later.