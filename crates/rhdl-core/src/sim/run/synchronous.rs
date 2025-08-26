use crate::rhdl_core::{
    trace, trace_time, ClockReset, RHDLError, Synchronous, SynchronousIO, TimedSample,
};

#[must_use = "To run the simulation, you must exhaust the iterator or collect it into a VCD"]
pub struct RunSynchronous<'a, T, I, S> {
    uut: &'a T,
    inputs: I,
    state: Option<S>,
    time: u64,
}

impl<T, I, S> Clone for RunSynchronous<'_, T, I, S>
where
    I: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        RunSynchronous {
            uut: self.uut,
            inputs: self.inputs.clone(),
            state: self.state.clone(),
            time: self.time,
        }
    }
}

pub fn run_synchronous<T, I, S>(uut: &T, inputs: I) -> RunSynchronous<'_, T, I, S> {
    RunSynchronous {
        uut,
        inputs,
        state: None,
        time: 0,
    }
}

impl<T, I, S> Iterator for RunSynchronous<'_, T, I, S>
where
    T: Synchronous<S = S>,
    I: Iterator<Item = TimedSample<(ClockReset, <T as SynchronousIO>::I)>>,
{
    type Item = TimedSample<(ClockReset, <T as SynchronousIO>::I, <T as SynchronousIO>::O)>;

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
            trace("clock", &sample.value.0.clock);
            trace("reset", &sample.value.0.reset);
            let output = self.uut.sim(sample.value.0, sample.value.1, state);
            Some(sample.map(|(cr, i)| (cr, i, output)))
        } else {
            None
        }
    }
}

pub trait RunSynchronousExt<I>: Synchronous + Sized {
    fn run(
        &self,
        iter: I,
    ) -> Result<
        RunSynchronous<'_, Self, <I as IntoIterator>::IntoIter, <Self as Synchronous>::S>,
        RHDLError,
    >
    where
        I: IntoIterator;
}

impl<T, I> RunSynchronousExt<I> for T
where
    T: Synchronous,
    I: IntoIterator<Item = TimedSample<(ClockReset, <T as SynchronousIO>::I)>>,
{
    fn run(
        &self,
        iter: I,
    ) -> Result<
        RunSynchronous<'_, Self, <I as IntoIterator>::IntoIter, <Self as Synchronous>::S>,
        RHDLError,
    > {
        self.yosys_check()?;
        Ok(run_synchronous(self, iter.into_iter()))
    }
}

pub trait RunWithoutSynthesisSynchronousExt<I>: Synchronous + Sized {
    fn run_without_synthesis(
        &self,
        iter: I,
    ) -> Result<
        RunSynchronous<'_, Self, <I as IntoIterator>::IntoIter, <Self as Synchronous>::S>,
        RHDLError,
    >
    where
        I: IntoIterator;
}

impl<T, I> RunWithoutSynthesisSynchronousExt<I> for T
where
    T: Synchronous,
    I: IntoIterator<Item = TimedSample<(ClockReset, <T as SynchronousIO>::I)>>,
{
    fn run_without_synthesis(
        &self,
        iter: I,
    ) -> Result<
        RunSynchronous<'_, Self, <I as IntoIterator>::IntoIter, <Self as Synchronous>::S>,
        RHDLError,
    >
    where
        I: IntoIterator,
    {
        Ok(run_synchronous(self, iter.into_iter()))
    }
}
