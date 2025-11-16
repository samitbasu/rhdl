//! Extension trait and types to provide for iterator-based open loop testing of
//! asynchronous circuits.
use crate::{Circuit, CircuitIO, TimedSample, trace_time};

/// An iterator that runs an asynchronous circuit given an iterator of timed inputs.
///
/// Generally, you will not construct this type directly, but instead use the
/// [`RunExt::run`] extension method on the circuit under test.
#[must_use = "To run the simulation, you must exhaust the iterator or collect it into a VCD"]
pub struct Run<'a, T, I, S> {
    uut: &'a T,
    inputs: I,
    state: Option<S>,
    time: u64,
}

impl<T, I, S> Clone for Run<'_, T, I, S>
where
    I: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Run {
            uut: self.uut,
            inputs: self.inputs.clone(),
            state: self.state.clone(),
            time: self.time,
        }
    }
}

/// Extension trait to provide a `run` method on asynchronous circuits.
///
/// This trait is automatically implemented for all types that implement
/// [`Circuit`].
///
/// See the book for examples of how to use this trait.
pub fn run<T, I, S>(uut: &T, inputs: I) -> Run<'_, T, I, S> {
    Run {
        uut,
        inputs,
        state: None,
        time: 0,
    }
}

impl<T, I, S> Iterator for Run<'_, T, I, S>
where
    T: Circuit<S = S>,
    I: Iterator<Item = TimedSample<<T as CircuitIO>::I>>,
{
    type Item = TimedSample<(<T as CircuitIO>::I, <T as CircuitIO>::O)>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get a mutable borrow to the state.  If the state is None
        // then initialize it first.
        let state = self.state.get_or_insert_with(|| self.uut.init());
        if let Some(sample) = self.inputs.next() {
            assert!(
                sample.time >= self.time,
                "input time must be non-decreasing"
            );
            self.time = sample.time;
            trace_time(sample.time);
            let output = self.uut.sim(sample.value, state);
            Some(sample.map(|i| (i, output)))
        } else {
            None
        }
    }
}

/// Extension trait to provide a `run` method on asynchronous circuits.
pub trait RunExt<I>: Circuit + Sized {
    /// Runs the circuit with the given iterator of timed inputs.
    fn run(&self, iter: I) -> Run<'_, Self, <I as IntoIterator>::IntoIter, <Self as Circuit>::S>
    where
        I: IntoIterator;
}

impl<T, I> RunExt<I> for T
where
    T: Circuit,
    I: IntoIterator<Item = TimedSample<<T as CircuitIO>::I>>,
{
    fn run(&self, iter: I) -> Run<'_, Self, <I as IntoIterator>::IntoIter, <Self as Circuit>::S> {
        run(self, iter.into_iter())
    }
}
