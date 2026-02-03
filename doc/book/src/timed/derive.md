# Derive

As `Timed` is a marker trait you need to implement it on any type you construct that will be passed in or out of a `Circuit`.    Recall from the foundation diagran of a circuit:

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

We can augment this diagram with a couple of additional types, which are defined by the `CircuitIO` trait:

```badascii
       +----------------------------------------------------------------+        
       |   +--+ CircuitIO::I                   CircuitIO::O +--+        |        
 input |   v               +-----------------------+           v        | output 
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


The input to the circuit is of type `CircuitIO::I`, and the output is of type `CircuitIO::O`.  The definition of the `CircuitIO` trait is summarized here:

```rust
{{#rustdoc_include ../code/src/timed/derive.rs:circuit_io_trait}}
```

where the `Timed` trait is simply a marker on top of `Digital:

```rust
{{#rustdoc_include ../code/src/timed/derive.rs:timed_trait}}
```

Thus, the input and output types to a circuit must `impl Timed`.  

```admonish note
Stable Rust does not allow us to define our own auto-traits.  `Timed` would be a good candidate for an auto-trait, since if all the elements of a data structure are `Timed`, then the data structure as a whole is `Timed`.   For now, this is not possible, and you must manually `impl Timed`.
```

The simplest way to indicate that a data structure is `Timed` is to add it to the list of derived traits, like so:

```rust
{{#rustdoc_include ../code/src/timed/derive.rs:output_example}}
```

If you were to expand out the `derive(Timed)` macro it would literally generate something like this:

```rust
{{#rustdoc_include ../code/src/timed/derive.rs:timed_impl}}
```

which will cause an compiler error if one of the elements in your data structure is not `Timed`.  This is the safest way to mark a datastructure.  Of course, you can also simply

```rust
{{#rustdoc_include ../code/src/timed/derive.rs:timed_blanket_impl}}
```

but in this case the type system will not protect you from having a `!impl Timed` element in your data structure.  RHDL, however, will still flag this as an error later.

