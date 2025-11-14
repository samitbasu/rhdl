# Run

Recall from [here](../../circuits/simulation.md) that the `Circuit` trait included a `sim` method:

```rust
pub trait Circuit: 'static + CircuitIO + Sized {
    type S: Clone + PartialEq;

    fn init(&self) -> Self::S;
    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O;
    // snip
}
```

and that a simulation loop would need to look something like this to drive the simulation:

```rust
let uut = T::new(); // 👈 or however you get an instance...
let mut state = uut.init();
loop {
    let i = <next_input of type T::I>;
    let o = uut.sim(i, &mut state);
    // Report value of `o`
    // Decide when to stop
}
```

To facilitate tracing, we will also want to establish the time `t0` associtated with each input sample.  We do this with the [TimedSample](../time.md) struct, which associates a time with each input sample.  

Here is the extension trait that gives us the `run` method on our `Circuit`:

```rust
pub trait RunExt<I>: Circuit + Sized {
    fn run(&self, iter: I) -> Run<'_, Self, <I as IntoIterator>::IntoIter, <Self as Circuit>::S>
    where
        I: IntoIterator;
}
```

and the blanket implementation:

```rust
impl<T, I, S> Iterator for Run<'_, T, I, S>
where
    T: Circuit<S = S>,
    I: Iterator<Item = TimedSample<<T as CircuitIO>::I>>,
{
    type Item = TimedSample<(<T as CircuitIO>::I, <T as CircuitIO>::O)>;

    fn next(&mut self) -> Option<Self::Item> {
        // snip
    }
}
```

Essentially, these together say that:

- You have some thing `i` which implements `IntoIterator`, with an `Item = TimedSample<I>` where `I` is the circuit input type.
- You have a circuit `x` with input type `I` and output type `O`.
- Calling `x.run(i)` will yield a new iterator that yields items of type `TimedSample<(I,O)>`.

In practical terms, this means that if you can generate a sequence of timed input samples of type `I`, then the `x.run()` method will transform these into a sequence of timed samples of type `(I,O)`, which is quite handy for testing.

Similarly for `impl Synchronous`, the `RunSynchronousExt` trait:

```rust
/// Extension trait to provide a `run` method on synchronous circuits.
pub trait RunSynchronousExt<I>: Synchronous + Sized {
    /// Runs the circuit with the given iterator of timed inputs.
    fn run(
        &self,
        iter: I,
    ) -> RunSynchronous<'_, Self, <I as IntoIterator>::IntoIter, <Self as Synchronous>::S>
    where
        I: IntoIterator;
}
```

along with the blanket implementation:

```rust
impl<T, I> RunSynchronousExt<I> for T
where
    T: Synchronous,
    I: IntoIterator<Item = TimedSample<(ClockReset, <T as SynchronousIO>::I)>>,
{
    fn run(
        &self,
        iter: I,
    ) -> RunSynchronous<'_, Self, <I as IntoIterator>::IntoIter, <Self as Synchronous>::S>;
}
```

and

```rust
// snip
impl<T, I, S> Iterator for RunSynchronous<'_, T, I, S>
where
    T: Synchronous<S = S>,
    I: Iterator<Item = TimedSample<(ClockReset, <T as SynchronousIO>::I)>>,
{
    type Item = TimedSample<(ClockReset, <T as SynchronousIO>::I, <T as SynchronousIO>::O)>;
    // snip
}
```

Essentially, these together say that:

- You have some thing `i` which implements `IntoIterator`, with an `Item = TimedSample<(ClockReset, I)>` where `I` is the circuit input type.  Clock and reset information are contained in the `ClockReset` part of the tuple.
- You have a synchronous circuit `x` with input type `I` and output type `O`.
- Calling `x.run(i)` will yield a new iterator that yields items of type `TimedSample<(ClockReset,I,O)>`.

In practical terms, this means that if you can generate a sequence of timed input samples of type `(ClockReset, I)`, then the `x.run()` method will transform these into a sequence of timed samples of type `(ClockReset, I,O)`.


