# With Reset
Recall that a synchronous circuit has a couple of additional input signals, beyond the input and output:

```badascii
        +---------------------------------------------------------------+        
        |   +--+ SynchronousIO::I           SynchronousIO::O +--+       |        
  input |   v               +-----------------------+           v       | output 
 +----->+------------------>|input            output+-------------------+------->
        | +---------------->|c&r      Kernel        |                   |        
        | |            +--->|q                     d+-----+             |        
        | |            |    +-----------------------+     |             |        
        | |            |                                  |             |        
        | |            |    +-----------------------+     |             |        
        | |q.child_1+> +----+o        child_1      i|<----+ <+d.child_1 |        
        | +-----------+|+-->|c&r                    |     |             |        
        | |            |    +-----------------------+     |             |        
        | |            |                                  |             |        
  clock | |            |    +-----------------------+     |             |        
& reset | |q.child_2+> +----+o        child_2      i|<----+ <+d.child_2 |        
 +------+-+---------------->|c&r                    |                   |        
  (c&r) |                   +-----------------------+                   |        
        +---------------------------------------------------------------+        
```

A `Synchronous` circuit includes a `ClockReset` input which provides the clock that governs the transitions of the circuit, and a reset line to set the state to some known initial value.  A `Synchronous` circuit also automatically fans those signals out to all of the internal child circuits of the main circuit.  

When working with `Synchronous` circuits, then, you need a different trait to do open-loop iterator based testing.  Here is the extension trait:

```rust
{{#rustdoc_include ../../code/src/simulations.rs:run_synchronous_ext}}
```

It provides the same `.run` method we used for asynchronous circuits [here](../iterator.md).  The blanket implementation illustrates the critical difference, though:

```rust
{{#rustdoc_include ../../code/src/simulations.rs:run_synchronous_ext_impl}}
```

Unlike the asynchronous case, the synchronous case requires a sequence of `TimedSample<(ClockReset, I)>` inputs.  This means we cannot simply feed a sequence of inputs into `.uniform()` and then simulate the circuit.  We _must_ provide the clock and reset signals.  There are a number of extension traits that are meant to help with this process.

## With Reset

Suppose we have an iterator `i` that yields items `i0, i1, i2...` each of type `I`, where `I` is the `SynchronousIO::I` input type of the circuit.  We would like to start out by sending a reset pulse to our circuit, and then the data items.  So we want the input to the circuit to look something like this:

```badascii
+reset+-+i0+-+-+i1+-+-+i2+-+    
+-----+------+------+------+ ...
   p0    p1     p2     p3       
```

where `p0, p1, p2` are the successive clock periods.  In RHDL, there is a data type that is similar in shape to `Option<T>` called `ResetOrData<T>`.  The full definition of the enum is here:

```rust
{{#rustdoc_include ../../code/src/simulations.rs:reset-or-data}}
```

```admonish note
While I experimented with using `Option` for this type, it lead to less readable code, so stuck with this somewhat verbose type.  It makes test code easier to read.
```

Given the iterator `i` that yields `i0, i1, ...`, we can construct the reset and data sequence as roughly:

```rust
{{#rustdoc_include ../../code/src/simulations.rs:reset_then_data}}
```

If we need `N` reset pulses (sometimes needed if the reset itself needs to cross clock domains or if some device requires a multi-clock reset for its internal circuitry), then we could use

```rust
{{#rustdoc_include ../../code/src/simulations.rs:reset_N_then_data}}
```

If _no_ reset is required/desired, then we can simply:

```rust
{{#rustdoc_include ../../code/src/simulations.rs:no-reset}}
```

The `ResetExt` trait and the corresponding blanket implementation mean that we can simply all

```rust
{{#rustdoc_include ../../code/src/simulations.rs:with-reset-usage}}
```

to generate an iterator that will insert the `N` reset pulses before feeding the iterator data.  Alternately, if you want `N=0`, then you can use the method:

```rust
{{#rustdoc_include ../../code/src/simulations.rs:without-reset-usage}}
```
