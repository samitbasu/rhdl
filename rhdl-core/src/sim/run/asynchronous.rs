use crate::{trace_time, Circuit, CircuitIO, TimedSample};

#[must_use = "To run the simulation, you must exhaust the iterator or collect it into a VCD"]
pub struct Run<'a, T, I, S> {
    uut: &'a T,
    inputs: I,
    state: Option<S>,
    time: u64,
}

impl<'a, T, I, S> Clone for Run<'a, T, I, S>
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

pub fn run<T, I, S>(uut: &T, inputs: I) -> Run<'_, T, I, S> {
    Run {
        uut,
        inputs,
        state: None,
        time: 0,
    }
}

impl<'a, T, I, S> Iterator for Run<'a, T, I, S>
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

pub trait RunExt<I>: Circuit + Sized {
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
