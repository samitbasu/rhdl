# Loops

In general, kernel's do not loop.  Inside your kernel, you can only use a very limited form of looping.  The only supported version of looping in a kernel is a `for` loop with the following constraints:

1. The loop bounds must be simple and computable at compile time.
2. The loop index must be an integer.
3. No early exit (e.g., `break`/`continue`) are allowed.
4. The `for` loop must iterate over a half open range with both ends defined, e.g., `for ndx in 3..9`.

The best way to think of a `for` loop in a hardware design is as a shorthand for copy-pasting a given design block a (fixed and predetermined) number of times.  In order to translate the loop correctly, RHDL needs to determine exactly how many times you want the block repeated.  It then "unrolls" the loop by creating the requested number of copies of the internal design.

A good example of using a `for` loop is to count the number of bits set in a given value.  The kernel for that might look something like this:

```rust
{{#rustdoc_include ../code/src/kernels/loops.rs:step_1}}
```

This simple looking kernel will generate quite a monstrous tree of adders in a long chain.  Something like this:

```badascii
 b0    +---+                  
+----->|   |    +---+         
 b1    | + +--->|   |    +---+
+----->|   |    | + +--->|   |
 b2    +---+    |   |    |   |
+-------------->|   |    | + |
 b3             +---+    |   |
+----------------------->|   |
                         +---+
            .                 
            .                 
            .                 
```

where `b0..` are the bits of `a`.  You can also make the loop depend on the generic parameter (which makes it more useful).  Suppose, for example, we want a generic ones-counter that will work for any bit vector width.

```rust
{{#rustdoc_include ../code/src/kernels/loops.rs:step_2}}
```

This version of the kernel will adapt to the `const` generic parameters `N` and `M`.  

Using a loop does not necessarily imply a long linear chain of circuitry.  The linear chain nature of the ones-counter is due to the need for the count to propagate from one iteration of the loop to another.  You can also do parallel operations using a `for` loop.  Here is an example that builds a XNOR gate.

```rust
{{#rustdoc_include ../code/src/kernels/loops.rs:step_3}}
```

After optimization, this circuit is completely parallel.  Each bit of the output can be computed from the corresponding bit of `a` and `b`, which means that there is no long chain of logic from the inputs to the output.

You can also use `for` loops to deal with arrays.  For example, the previous kernel could also be written as follows, where `a, b` have been pre-expanded into bool arrays:

```rust
{{#rustdoc_include ../code/src/kernels/loops.rs:step_4}}
```

Here the parallelism is clear.