# Timing Estimation

## Baseline Performance

While we have a design that works, it is not particularly efficient.  If you were to diagram out our ones counter, it would look something like this:

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

where each successive bit is added to the output of the previous one.  This deep chain can create a design that runs slowly, since the change of the signal `b0` has a long way to propagate before it reaches the final `sum` output.  To see that this is the case, we can use another method of the `IceStorm` toolchain.  This one takes a design and computes the longest path through it without actually building a binary or flashing the design to an FPGA.  It gives us a simple way to estimate the timing propagation through a RHDL design.  We will put it into a test case for ease of access later.

```rust
{{#rustdoc_include ../code/src/count_ones.rs:ones-step-12}}
```

Running this test shows that the time to propogate through our circuit is roughly 11 nanoseconds:

<!-- cmdrun to-html "cd ../code && cargo test --package code --lib -- count_ones::step_4::test_base_timing --exact --nocapture" -->

While currently, that doesn't matter, in the future, we may want to put this design into a clocked circuit.  With an 11 nanosecond delay, it means that we wont be able to run that clocked circuit particularly quickly.  Furthermore, there isn't anything an optimizer can really do at this point.  The design has already been optimized by `yosys`.  To improve the performance, we need to rethink how we count the ones in the first place.  RHDL gives us tools to do hardware design, but does not eliminate the need to think!  Ideally, it just makes it easier to translate our ideas into working code with fewer surprises.

## Divide and Conquer

The simplest strategy to flatten the design is to divide and conquer.  Calculating the sum of ones is parallelizable over the bits in the input vector.  So we should be able to simply split the sum into two parts:

```badascii
          +-----+                 
  b0..b3  | cnt |                 
+-------->| one +---+   +-----+   
          |     |   |   |     |   
          +-----+   +-->|     |   
                        | sum +-->
          +-----+   +-->|     |   
  b4..b7  | cnt |   |   |     |   
+-------->| one +---+   +-----+   
          |     |                 
          +-----+                 
```

Each of the 4-bit one-counters will have half the tree depth of the 8-bit one-counter, and should thus give a better performance.  So we will try to split up our ones-counter into a pair of smaller counters and then add the results.

## Pure Functions

To make it easier to compare designs, we will first move the existing logic into it's own function, and make it generic over the size of the bitvector being summed.  Let us put the following function into `helper.rs` and add the `pub mod helper` line to `lib.rs` so that we include it.

```rust
{{#rustdoc_include ../code/src/count_ones.rs:ones-step-13}}
```

There are some new things introduced here.  First, we have a `#[kernel]` that has a different type signature than our `fn(I,Q) -> (O, D)`.  This is because you can generally write any pure function as a `kernel` by adding the `#[kernel]` annotation to the signature.  The second is that we have made the function generic over the bitwidth of the input and output bitvectors.  This would allow us to use it parametrically throughout our designs.  Because it is just a plain Rust function, we can still test it with a simple test:

```rust
{{#rustdoc_include ../code/src/count_ones.rs:ones-step-14}}
```

We can in fact, rewrite our top level kernel to just call the helper function.  With that change, we have the following for `lib.rs`:

```rust
{{#rustdoc_include ../code/src/count_ones.rs:ones-step-15}}
```

Now we can create a new module where the circuit uses subdivision to compute the number of ones.

```rust
{{#rustdoc_include ../code/src/count_ones.rs:ones-step-16}}
```

We can now test it using the same method as the before:

```rust
{{#rustdoc_include ../code/src/count_ones.rs:ones-step-17}}
```

Rerunning the timing calculation is now as simple as making a new test

```rust
{{#rustdoc_include ../code/src/count_ones.rs:ones-step-18}}
```

If we run this test, we get an updated timing result

<!-- cmdrun to-html "cd ../code && cargo test --package code --lib -- count_ones::step_6::test_timing_divided --exact --nocapture" -->

We now have a timing of ~10 nsec, which is somewhat faster.  You can flash this design and test it out as an exercise.

You can take it one more level, but the routing delay dominates at this point. 

```rust
{{#rustdoc_include ../code/src/count_ones.rs:ones-step-19}}
```

With an updated test:

```rust
{{#rustdoc_include ../code/src/count_ones.rs:ones-step-20}}
```

<!-- cmdrun to-html "cd ../code && cargo test --package code --lib -- count_ones::step_6::test_timing_divided_four --exact --nocapture" -->

