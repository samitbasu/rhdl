Here are the comments from the reviewers:


Review 1A:

Throughout the paper there isn't much of a comparison with other options, and with all new languages, it is useful to understand what a user would gain over the potentially more established alternatives?

DONE - Address this point.

Review 1B:

The descriptions of using RustHDL for hardware vs. firmware are a little confusing and unclear.  It is clear that it generates Verilog, but is that described interchangeably as both hardware and firmware? 

DONE - Clarify the terms "hardware and firmware".


It's also unclear to me how much of the work of understanding the design intent falls onto something like a generator framework vs. a compiler directly parsing the source code.  
 
DONE - Clarify generator framework vs. compiler intent.

 
The mission and goals of the project are worthwhile, and using Rust to describe hardware seems like an interesting approach.  The open discussion on feedback from users is fascinating and refreshing.  This is an interesting paper that leaves me wanting to hear more about it, ask more questions, and have a discussion about it.

Review 1C: 

I love to see new HDLs being developed and I am also a big fan of Rust. A couple of comments that might help you improve your paper:

It sounds to me like the current version of RustHDL is more similar to Chisel and Amaranth which generate a cricuit description as the Scala / Python code is executed than to MyHDL which - as far as I know - analyzes the Python AST and would thus be more similar to your proposed RHDL language.

DONE - Add a comment about how the circuit description is generated from the AST, even in RustHDL.


It would be great if you could show code for the following things:
- instantiating a module
- wiring up modules (the `join` statement)

DONE - Add an example of instantiating a module
DONE - Add an example of `join`.


The section on the `Mental Model` is  very confusing to me. What are the connection semantics? I guess they are similar to `firrtl`'s (and thus Chisel's) "last connect" semantics, meaning that if a signal is connected multiple times, the last connection wins. Is that true? You also only show connections to the `next` value of a register. How would you connect to a `Local` `Signal`? Can you connect to a local signal _after_ reading it?

DONE - Explain the mental model.


It would be nice to see a simulation speed comparison of a design using the native event-driven simulator vs. generating the Verilog and executing it with Verilator.

DONE - Add comment about Verilator and performance.

Could you elaborate on how your even-driven simulator works if there are combinatorial paths across modules? Does it execute `update` functions until there is a fixed point?

DONE - Answer this question.

I am also curious to know whether any meta-programming facilities are supported. Like, can you generate a bus topology from a declarative description? 

DONE - Provide example of meta programming.

I also wonder how meta-programming would interact with your proposed switch to directly working on the AST. How would you know which code will be executed at elaboration vs. at circuit run time?

TODO - Good question.


Review 1D:

* Some missing related work: ShakeFlow is another relevant Rust-based language
  for hardware design.

TODO - Add reference and comments.

* I think in the presentation at LATTE, it would benefit the argument to point
  out specific Rust features and how they benefit hardware development. Use
  examples, e.g., catching a bug early via Rust's type checking (that maybe
  would not be so obvious in a non-Rust HDL).  I agree with the author that Rust
  has powerful features and robust tooling, but hardware developers coming from
  more conventional HDLs may not see the benefit from reading Rust's feature
  list alone. The argument can be strengthened by showing explicitly how Rust
  can help with specific hardware development challenges.

TODO - Address in the presentation.

* With the examples in the paper, it's not clear how a Rust user without
  hardware experience would easily implement a design in RustHDL/RHDL. This
  point should be addressed in the presentation.

TODO - Address in the presentation.

  Review 1E:
  In general I think this paper lacks focus. At times it's hard to figure out if you are discussing RustHDL or RHDL, and sometimes you mentions positive qualities mixed in with the negative qualities. In any case, I do not think discussing disadvantages of an existing language is beneficial. The paper I would like to see is after your work on RHDL, and showing what was improved and how. Notwithstanding, I do expect more detailed comparison with at least the alternatives cited like Spade and XLS, but since RHDL is slated to be a DSL, then also to other languages like Chisel and Amaranth.

TODO - ??