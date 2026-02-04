# The Counter

Here is the counter we need to build:

```badascii
      +--------------------------------> Overflow
      |    1                                     
      |    +                                     
      |    v                                     
carry |  +-----+                                 
      +--+ add |                                 
         |  w  |                                 
 sum  +-+|carry|<---+------------+               
      |  +-----+    |            |               
      |             |            |               
 +---+|+------------+            |               
 |    |                          |               
 |    |     +                    |               
 |    |     |\        +-+DFF+-+  |               
 |    +---->|1+       |       |  |               
 |          | |++---->|D     Q+--+------> Output 
 +--------->|0+       |       |                  
            |/        |       |                  
            +^        |cr     |                  
   Enable    |        +-------+                  
  +----------+         ^                         
   ClockReset          |                         
  +--------------------+                         
```

Our counter will be a `struct` in Rust.  Even though it has a MUX and an adder, we won't put these in as components (although you absolutely could).  Instead, because these elements are not explicitly clocked, we will infer them using normal Rust code in the form of an expression and an `if` statement.  We _do_ need an element to hold our count value, and that is provided by the `rhdl-fpga` crate.   So if you will need to add that to your dependency list.

```shell
cargo add rhdl-fpga
```

We can start with the skeleton of our counter:

```rust
{{#rustdoc_include ../code/src/blinky.rs:step-1}}
```

Next, we need to add the derive macro to turn this into a `Synchronous` circuit.  The appropriate derive incantations are:

```rust
{{#rustdoc_include ../code/src/blinky.rs:step-2}}
```

The `Sycnhronous` derive macro is similar to the `Circuit` derive macro we have seen before, but differs in a few important ways:

- The `Synchronous` trait will automatically inject the `ClockReset` signal into the circuit kernel and into each of the subcomponents.  
- As a result, the `Kernel` signature for a `Synchronous` circuit includes the `ClockReset` signal as the first argument.  More on that a bit later.
- The `Count8D` and `Count8Q` types are auto derived, just like they were with `CircuitDQ`.  And there is no substantial difference for these.
- The `Synchronous` trait requires that `Count8 impl SynchronousIO`.  We will get to this next.

Now we need to implement the `SynchronousIO` trait for our counter.  This trait defines the input and outputs of the circuit, similar to how we defined `CircuitIO` before.  The difference is that we do not need to deal with `Signal<_, _>` types here.  Instead, we can use plain `Digital` types, because all inputs and outputs are assumed to be in the timing domain defined by the clock signal that feeds the circuit.

For our counter, we want a input enable signal.  Now while we could use a raw boolean, it makes the code harder to read later.  So we will go ahead and create an input struct that has a single `enable` field as input.  While we are at it, we will also create the output struct, which has the `count` and `overflow` fields.

```rust
{{#rustdoc_include ../code/src/blinky.rs:step-3}}
```

```admonish note
It's best practice to put components into their own modules.  So that the input struct can just be called `I`, and you can refer to it as `counter::I` if the module is named `counter`.  This keeps things separated, so that you don't end up with name collisions between different components that name their inputs `I`.
```

