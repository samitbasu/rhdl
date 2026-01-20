# Inputs and Outputs

Completely analogous to the case for [CircuitIO](../circuits/circuit_io.md), the inputs and outputs for a Synchronous circuit are defined by a `SynchronousIO` trait.  Recall the foundational diagram for `Synchronous` circuits:

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

The `SynchronousIO` trait is short.  Here is the definition of that trait in its entirety:

```rust
{{#rustdoc_include ../code/src/synchronous.rs:synchronous-io}}
```

While it looks similar to the `CircuitIO` trait definition:

```rust
{{#rustdoc_include ../code/src/circuits/io.rs:circuit_io}}
```

there are a few differences, and they are all significant.  First, note that in the case of `Circuit`, `I: Timed`, and `O: Timed`.  And recall from [here](../circuits/circuit_io.md), that a `Timed` type is either:

- `Signal<T, D>` where `D: Domain` and `T: Digital`
- `()`
- Some tuple of `: Timed` types or an array `[T; N]` of them

On the other hand, note that in the `SynchronousIO` case:

```rust
{{#rustdoc_include ../code/src/synchronous.rs:synchronous-io-focus}}
```

This means that the input and output types are just `Digital`, and _not_ `Timed`.  Here the type system is telling us "no need to indicate the clock domain `D`, because this whole thing is synchronous to the provided clock...".  Because the circuit pinky-promises to change only on the clock edges of the provided clock, and because you promise to only feed it inputs that are synchronous that same clock, RHDL essentially removes all clock related bits from the type signatures of the inputs and outputs.

So while in a `Circuit`, you have inputs that are a bit clunky looking, like `Signal<bool, Red>`, in a `Synchronosu`, you can just have `bool`.  The `Signal<_, Red>` is implied, and assumed to be the same as the clock being `Signal<Clock, Red>` (and the reset too).  The same is true for the outputs.  Referring to the simple XOR gate example [here](../xor_gate/the_gate.md), the `CircuitIO` impl is somewhat clunky looking:

```rust
{{#rustdoc_include ../code/src/circuits/io.rs:circuit_io}}
```

And here, we took a shortcut.  Really an `XorGate` should be usable in any clock domain, which means, that it should really be generic over the clock domain `D`.  So....

```rust
{{#rustdoc_include ../code/src/synchronous.rs:xor-io-generic}}
```

It's not getting any better.  While explicit, and type checked, it is getting harder and harder to read.  A synchronous Xor gate (if there were such a thing) would instead have the following trait implementation

```rust
{{#rustdoc_include ../code/src/synchronous.rs:xor-sync-io}}
```

Before you decide that you will only use synchronous circuits, just remember that reality is _not_ synchronous.  Sooner or later you will have to deal with the clock domain or asynchrony of the inputs.

The other change is in the form of the compute kernel.  For `CircuitIO`, we had:

```rust
{{#rustdoc_include ../code/src/circuits/io.rs:circuit_io}}
```

For the `Kernel` associated type, we can state in words that the `kernel` is a synthesizable function (marked with `#[kernel]`) that has the type signature `fn(I, Q) -> (O, D)`.  For a synchronous circuit, the only difference is that the clock and reset are passed as the first argument:

```rust
{{#rustdoc_include ../code/src/synchronous.rs:synchronous-io}}
```

so that the `kernel` function is of the form `fn(ClockReset, I, Q) -> (O, D)`.  Having the `reset` available to the `kernel` is critical for implementing reset behavior.  The `clock` is less useful, but it's there if for some reason you need to do something with it.


```admonish note
You can probably commit crimes by using the `clock` input to the kernel to create long and nasty outputs from a synchronous circuit that violate setup and hold.  RHDL won't complain (because it does not model circuitry at that level of detail).  But hopefully your toolchain admonishes you severely.  Or forces a comically slow clock frequency.  Or writes a strongly worded e-mail to your parents.
```

Just as in the case of `Circuit`, the `kernel` arguments are always:
The signature of the kernel (for a `Circuit`) is always

```badascii
                  clock                    internal              internal 
                  reset                    feedback              feedback
                    +                         +                      +   
                    v                         v                      v   
                                                           
pub fn kernel(cr: ClockReset, i: Self::I, q: Self::Q) -> (Self::O, Self::D)
                                                           
                                    ^                         ^             
                                    +                         +             
                                 circuit                   circuit          
                                 inputs                    outputs          
```


