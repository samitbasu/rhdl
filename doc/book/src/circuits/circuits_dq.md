# Feedback

The foundational diagram includes two types `D` and `Q` that represent the feedback path in the circuit:

```badascii
            I                                                  O                 
            +                                                  +                 
       +---+|+------------------------------------------------+|+-------+        
       |    |                                                  |        |        
 input |    v              +-----------------------+           v        | output 
+----->+------------------>|input            output+--------------------+------->
       |                   |         Kernel        |                    |        
       |              +--->|q                     d+-----+              |        
       |   Q+-------->|    +-----------------------+     |<-----+D      |        
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

The feedback path is critical to building non-trivial circuitry using RHDL's structural pattern.  The `kernel` function takes the output of the internal circuitry of type `Q`, combines it with the current input, and generates the output `O` and the inputs to the internal circuitry of type `D`.  As this new input could cause the child circuits to update their outputs `Q`, the circuit will continue to change state until it settles to a new fixed point in which values are changing.  


```admonish note
If this seems strange, well, it is a little.  But real circuitry behaves _exactly_ this way.  Changes propagate through the child circuits, and those in turn cause the outputs to change.  Only when a new fixed point is reached does the circuit stop changing.  _Most_ of the time, you will work with Synchronous circuits, which tend to have limited propagation of changes, and no coupled feedback loops, because stateful components, like flip flops, will isolate their outputs from their inputs.  And circuits with complete combinatorial loops, in which inputs and outputs are coupled directly, are not allowed.  Even if they simulate in RHDL, eventually, your toolchain somewhere will throw an error.  
```

The `CircuitDQ` trait is pretty simple, and only tells part of the story:

```rust
{{#rustdoc_include ../code/src/circuits/dq.rs:circuit-dq}}
```

It explicitly requires that we define the two types `D` and `Q`, and that they `impl Timed`.  In general a `Timed` type is either a `Signal<T, D>` where `T: Digital` and `D: Domain` or `()`.

However, the trait does not provide the full set of constraints on the `D` and `Q` types.   And it is a design choice that leaves me conflicted.  For RHDL to actually function, the `D` and `Q` types must match the input and output types of the subcircuits.  And furthermore, when using the generated `Circuit` implementation, the types must be further constrained so that:

- Each type `D` and `Q` contains exactly one field with the same name as the corresponding child subcircuit for which we are implementing `Circuit`.
- For `D`, the field must correspond to the _input_ type of that subcircuit.
- For `Q`, the field must correspond to the _output_ type of that subcircuit.

It's easier to illustrate than explain.  Suppose we have a circuit that contains 3 subcircuits of type `A`, `B` and `C`, and suppose that these circuits are members of the parent circuit `X` with field names `child_1`, `child_2` and `child_3`.  So the declaration of `X` looks like

```rust
{{#rustdoc_include ../code/src/circuits/dq.rs:circuit-x}}
```

In this case, the type of `D` must be equivalent to:

```rust
{{#rustdoc_include ../code/src/circuits/dq.rs:circuit-x-d}}
```

and similarly, the type of `Q` must be equivalent to

```rust
{{#rustdoc_include ../code/src/circuits/dq.rs:circuit-x-q}}
```

There is a macro that automatically derives these exact type definitions, and you can simply add it to the list for `X`:

```rust
{{#rustdoc_include ../code/src/circuits/dq.rs:circuit-x-derive}}
```

This will cause RHDL to derive a pair of structs named `D` and `Q` and give them the definitions described above (with the appropriate generics as needed).

```admonish note
It might seem like `D` and `Q` should have just been defined as tuples, so that `D = (child_1::I, child_2::I, ...)` and similarly `Q = (child_1::O, child_2::O, ...)`.  And while from a Rust idiomatic perspective, these definitions may be the best, they do not lead to particularly clean kernels.  The approach I adopted here is messier because there is an implicit requirement on `D` and `Q` that is not otherwise expressed.  An alternate strategy would have been to define a pair of _traits_ and then `impl` the traits on structs.  I'm not sure the extra complexity is really worth it.  But I acknowledge that this aspect of the implementation is not particularly elegant.
```

