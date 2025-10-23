# Kernels

Almost all designs using RHDL involve the writing of `kernel` functions.  These are (almost) pure functions that use a subset of Rust syntax to describe the flow and transformation of signals through the internals of a circuit.  Recall from the foundational diagram:

```badascii
       +----------------------------------------------------------------+        
       |                                                                |        
 input |                   +-----------------------+                    | output 
+----->+------------------>|input            output+--------------------+------->
       |                   |         Kernel        |                    |        
       |              +--->|q                     d+-----+              |        
       |              |    +-----------------------+     |              |        
       |              |                                  |              |        
       |              |    +-----------------------+     |              |        
       | q.child_1 +> +----+o        child_1      i|<----+ <+ d.child_1 |        
       |              |    +-----------------------+     |              |        
       |              |                                  |              |        
       |              |    +-----------------------+     |              |        
       | q.child_2 +> +----+o        child_2      i|<----+ <+ d.child_2 |        
       |                   +-----------------------+                    |        
       |                                                                |        
       +----------------------------------------------------------------+        
```

The `kernel` is a function with the following high level characteristics:

1. It is (almost) pure.  The only side effects allowed are calls to the `trace` subsystem, which is used to log values inside the kernel as it executes.
2. It is a Rust function.  This is extremely important!  Every `kernel` must be a valid Rust function.  Not all valid Rust functions are kernels, but every valid `kernel` _must_ be a valid Rust function.
3. It has a type signature determined by the structure of the circuit.
4. It must be synthesizable.  There is a limited subset of Rust syntax that can be processed by RHDl into a hardware circuit description.  Using elements outside this subset result in functions that are not valid kernels.
5. It is stateless.  The `kernel` is pure, and thus, it cannot have internal state.  Internal state of a circuit is implemented by incorporating stateful elements.  
6. It is decorated with a `#[kernel]` attribute.
7. It can be tested as just a plain Rust function, or after partial or complete synthesis.

Once you have written a few `kernel` functions, you will quickly get the hang of it.  They are surprisingly easy and satisfying to write.  They describe in code how the circuit inputs and child circuit outputs are transformed to form circuit outputs and child circuit inputs.  We will cover many examples of `kernel` functions.

```admonish note
RHDL makes heavy use of proc macros.  It's necessary given what I am trying to do to the language.  However, unlike other proc-macro based systems, RHDL is _not_ a DSL.  Instead, if you remove the `#[kernel]` attribute, you are left with a regular Rust function.  An LSP like Rust Analyzer will provide you all the usual comforts of coding in Rust, and things like type inference, and precise errors are all available.  In fact, I generally write kernel functions _without_ the attribute, and then when they are correct, I add the attribute.  It makes for a much nicer development experience as we will see.
```

In this section, we will break down all of the syntax elements that you can use to write synthesizable code in RHDL, and highlight what Rust concepts are not supported as well.


```shell,rhdl-silent
rm -rf kernels
cargo new --lib kernels
cd kernels
cargo add --path ~samitbasu/Devel/rhdl/crates/rhdl 
cargo add --dev miette
```