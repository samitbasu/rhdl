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
pub trait CircuitIO: 'static + Sized + Clone + CircuitDQ {
    /// The input type of the circuit
    type I: Timed;
    /// The output type of the circuit
    type O: Timed;
    /// snip...
}
```

where the `Timed` trait is simply a marker on top of `Digital:

```rust
pub trait Timed: Digital {}
```

Thus, the input and output types to a circuit must `impl Timed`.  

```admonish note
Stable Rust does not allow us to define our own auto-traits.  `Timed` would be a good candidate for an auto-trait, since if all the elements of a data structure are `Timed`, then the data structure as a whole is `Timed`.   For now, this is not possible, and you must manually `impl Timed`.
```

The simplest way to indicate that a data structure is `Timed` is to add it to the list of derived traits, like so:

```rust
//                                          👇 - new!
#[derive(PartialEq, Digital, Clone, Copy, Timed)]
struct O {
    out1: Signal<b16, Red>,
    out2: Signal<b16, Red>,
    t_clk: Signal<Clock, Red>,
    t_out: Signal<b16, Red>,
    p_out: Signal<b16, Red>,
    bt_ready: Signal<bool, Red>,
}
```

If you were to expand out the `derive(Timed)` macro it would literally generate something like this:

```rust
impl rhdl::core::Timed for O
where
    Signal<b16, Red>: rhdl::core::Timed,
    Signal<b16, Red>: rhdl::core::Timed,
    Signal<Clock, Red>: rhdl::core::Timed,
    Signal<b16, Red>: rhdl::core::Timed,
    Signal<b16, Red>: rhdl::core::Timed,
    Signal<bool, Red>: rhdl::core::Timed,
{}
```

which will cause an compiler error if one of the elements in your data structure is not `Timed`.  This is the safest way to mark a datastructure.  Of course, you can also simply

```rust
impl Timed for O {}
```

but in this case the type system will not protect you from having a `!impl Timed` element in your data structure.  RHDL, however, will still flag this as an error later.

