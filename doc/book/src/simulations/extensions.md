# Synchronous Iterator Extensions
In [this](./iterator.md), we covered the basics of open-loop simulation of a circuit using iterators and the `.uniform` extension trait, along with the introduction of `TimedSample`.  The utility of iterator based testing for _asynchronous_ circuits tends to be limited, though, since there is no general notion of time in such circuits.  You can still do it, and often asynchronous circuits are composed of multiple synchronous ones running on different clocks.  But the iterator extensions really help with dealing with synchronous circuits.

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
pub trait RunSynchronousExt<I>: Synchronous + Sized {
    fn run(
        &self,
        iter: I,
    ) -> RunSynchronous<'_, Self, <I as IntoIterator>::IntoIter, <Self as Synchronous>::S>
    where
        I: IntoIterator;
}
```

It provides the same `.run` method we used for asynchronous circuits [here](./iterator.md).  The blanket implementation illustrates the critical difference, though:

```rust
impl<T, I> RunSynchronousExt<I> for T
where
    T: Synchronous,
    //                                     👇 different! 
    I: IntoIterator<Item = TimedSample<(ClockReset, <T as SynchronousIO>::I)>>,
{
    fn run(
        &self,
        iter: I,
    ) -> RunSynchronous<'_, Self, <I as IntoIterator>::IntoIter, <Self as Synchronous>::S>;
}
```

Unlike the asynchronous case, the synchronous case requires a sequence of `TimedSample<(ClockReset, I)>` inputs.  This means we cannot simply feed a sequence of inputs into `.uniform()` and then simulate the circuit.  We _must_ provide the clock and reset signals.  There are a number of extension traits that are meant to help with this process.

## With Reset

Suppose we have an iterator `i` that yields items `i0, i1, i2...` each of type `I`, where `I` is the `SynchronousIO::I` input type of the circuit.  We would like to start out by sending a reset pulse to our circuit, and then the data items.  So we want the input to the circuit to look something like this:

```badascii
+reset+-+i0+-+-+i1+-+-+i2+-+    
+-----+------+------+------+ ...
   p0    p1     p2     p3       
```

where `p0, p1, p2` are the successive clock periods.  One way to model this is to use the rust `Option<I>` type to indicate that we want data of type `I` or a reset.  We thus want to yield something like:

```badascii
+reset+-+i0+-+-+i1+-+-+i2+-+    
+-----+------+------+------+ ...
   p0    p1     p2     p3       
             +                  
             v                  
                                
++None++Some+++Some+++Some++    
|     | (i0) | (i1) | (i2) |    
+-----+------+------+------+ ...
   p0    p1     p2     p3       
```

This is simple to construct with iterators.  We would simply `once(None).chain(i.map(Some))`.  If we wanted more resets (like 3 reset intervals, instead of just the 1), we could use `repeat(None).take(3).chain(i.map(Some))`.  The shortcut for this is provided by the `with_reset` method and the `without_reset` methods from another extension trait:

```rust
/// Extension trait to provide `with_reset` and `without_reset` methods on iterators.
pub trait TimedStreamExt<Q>: IntoIterator + Sized
where
    Q: Digital,
{
    /// Creates a ResetWrapper that does not prepend any reset pulses.
    fn without_reset(self) -> ResetWrapper<<Self as IntoIterator>::IntoIter>;

    /// Creates a ResetWrapper that prepends the given number of reset pulses.
    fn with_reset(self, pulse: usize) -> ResetWrapper<<Self as IntoIterator>::IntoIter>;
}
```

Thus, we can call `i.with_reset(n)` which is equivalent to `repeat(None).take(n).chain(i.map(Some))` or `i.without_reset()`, which is equivalent to `i.map(Some)`.  In both cases, we turn an iterator with item `I` into an iterator of item `Option<I>`, with the `None` variant representing the `Reset` case, and the `Some` variant representing data.  This is not sufficient for simulation yet, as we still need to generate the `ClockReset` signals from this sequence of `Option<I>`.  That operation is provided by a slightly more complicated extension trait.





