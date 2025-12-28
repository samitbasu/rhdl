//! Extension trait and types to provide for iterator-based open loop testing of
//! synchronous circuits.
use crate::{
    ClockReset, Synchronous, SynchronousIO, TimedSample, trace,
    trace2::{
        page::{set_trace_page, take_trace_page},
        session::Session,
        trace_sample::TracedSample,
    },
};

/// An iterator that runs a synchronous circuit given an iterator of timed inputs.
#[must_use = "To run the simulation, you must exhaust the iterator or collect it into a VCD"]
pub struct RunSynchronous<'a, T, I, S> {
    uut: &'a T,
    inputs: I,
    state: Option<S>,
    time: u64,
    session: Session,
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
            session: self.session.clone(),
        }
    }
}

/// Runs the synchronous circuit with the given iterator of timed inputs.
pub fn run_synchronous<T, I, S>(uut: &T, inputs: I) -> RunSynchronous<'_, T, I, S> {
    RunSynchronous {
        uut,
        inputs,
        state: None,
        time: 0,
        session: Session::default(),
    }
}

impl<T, I, S> Iterator for RunSynchronous<'_, T, I, S>
where
    T: Synchronous<S = S>,
    I: Iterator<Item = TimedSample<(ClockReset, <T as SynchronousIO>::I)>>,
{
    type Item = TracedSample<(ClockReset, <T as SynchronousIO>::I), <T as SynchronousIO>::O>;

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
            let trace_page = sample.is_traced().then(|| self.session.page());
            set_trace_page(trace_page);
            trace("clock", &sample.value.0.clock);
            trace("reset", &sample.value.0.reset);
            let output = self.uut.sim(sample.value.0, sample.value.1, state);
            let page = take_trace_page();
            Some(TracedSample {
                input: sample.value,
                output,
                time: sample.time,
                page,
            })
        } else {
            None
        }
    }
}

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

impl<T, I> RunSynchronousExt<I> for T
where
    T: Synchronous,
    I: IntoIterator<Item = TimedSample<(ClockReset, <T as SynchronousIO>::I)>>,
{
    fn run(
        &self,
        iter: I,
    ) -> RunSynchronous<'_, Self, <I as IntoIterator>::IntoIter, <Self as Synchronous>::S> {
        run_synchronous(self, iter.into_iter())
    }
}
