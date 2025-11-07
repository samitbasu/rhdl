# Exhaustive Testing

In RHDL, "exhaustive testing" is fairly easy to do.  Recall from the canonical diagram:

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

that the `kernel` is a _pure function_ that maps the current set of child circuit outputs `q` and the input `input` into an output `output` and a new set of child circuit inputs `d`.  The function signature is something like:

```rust
fn kernel(i: Self::I, q: self::Q) -> (Self::O, Self::D);
```

for asynchronous circuits, and

```rust
fn kernel(cr: ClockReset, i: Self::I, q: self::Q) -> (Self::O, Self::D);
```

for synchronous circuits.  The functions must be pure, meaning no side effects (tracing is considered irrelevant here - the trace mechanism cannot change the result of computing the function as it is write only).  

What is interesting about this design, is that you can, in principle, exhaustively test `kernel` by presenting it all possible inputs and checking that the output is correct.  That may not be _practical_ in most cases, but it is certainly possible.  And for sure, you can choose those inputs that are most interesting or the most likely to expose issues in your implementation.

Remember that the `kernel` is just a Rust function.  So you can test it like any other Rust function.  To make the example concrete, suppose we have the kernel of a counter.  It has an enable signal as the only (boolean) input, and a set of digital flip flops that store the current count.  The kernel function would probably have a signature like this:

```rust
fn counter(cr: ClockReset, i: bool, q: b8) -> (b8, b8);
e```

(where we have assumed that the counter is 8 bits wide).  We can now test this function exhaustively.  The function itself shouldn't care about the clock or reset values (those are handled in the flip flops).  So we just want an invariant like:

-  Any time `i` is true, the feedback to the internal counter should be equal to `q+1` (modulo wrap around).
- Any time `i` is false, the feedback to the internal counter should be equal to `q`.
- The output is always equal to the current count value `q`.

These requirements can be coded into a test that looks something like:

```rust
#[test]
fn test_counter_exhaustively() {
    let cr = clock_reset(clock(false), reset(false));
    for i in [false, true] {
        for q in (0..256).map(b8) {
            let (o, d) = counter(cr, i, q);
            if i {
                assert_eq!(d, q + 1);
            } else {
                assert_eq!(d, q);
            }
            assert_eq!(o, q);
        }
    }
}
```

Note that there is nothing RHDL specific about the function or the test harness.  We are simply using the fact that the function is pure to ensure we can test 100% of the possible inputs and it could see, and then verify that in all cases, the expected behavior is observed.  This can be really powerful when the inner workings of the function are complicated, but the output and next state can be checked for correctness somewhat easily.

There are other scenarios where exhaustive testing might be useful. Consider a state machine, for example.  If we write it as a pure state machine, then `q` is the current state, `i` is the input, and `d` is the next state, with `o` being the output.  Because we can independently control the current state `q` and the input, we can check all possible state transitions on the function, which can be very difficult to do with normal testing.    State machines are often incompletely specified, with an implication that some inputs "cannot happen" when in some states, or the expectation of a default behvior.  For example, consider the following simple state machine:

```badascii
        +----+           
     +->|Init+-+         
     |  +----+ |         
     |         | start   
     |         |         
     |  +----+ |         
stop |  |Run |<+--+      
     |  +--+-+    |      
     |     | stop |      
     |     v      | start
     |  +-----+   |      
     +--+Pause+---+      
        +-----+          
```

This machine is underspecified (what do you do when you receive a `start` and are already in `Run`?), and some states may be unreachable from other states with only valid inputs.  Fortunately, with a pure update function, we can simply test all possible input and current state combinations, and then check that the output and next state match our expectations.  By separating the "business" logic that computes the next state and the output from the rest of the circuit, RHDL effectively allows you to test that logic exhaustively to ensure it is correct.  You may not need exhaustive testing much, but when a circuit's function is critical, you will be glad to be able to do it. 
