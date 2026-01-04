//! A timed sample with an (option) trace page attached.
//!
//! This is how you collect trace data when using the iterator based simulation API.

use crate::{
    Digital, TimedSample,
    trace::page::{TracePage, set_trace_page, take_trace_page},
};

/// A timed sample with an optional trace page attached.
pub struct TracedSample<T: Digital, S: Digital> {
    /// The time of the trace
    pub time: u64,
    /// The input value at this time
    pub input: T,
    /// The output value at this time
    pub output: S,
    /// The trace data for this sample belongs to.
    pub page: Option<Box<TracePage>>,
}

impl<T: Digital, S: Digital> std::fmt::Display for TracedSample<T, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "@{}: {:?} -> {:?}",
            self.time,
            self.input.typed_bits(),
            self.output.typed_bits()
        )
    }
}

impl<T: Digital, S: Digital> TracedSample<T, S> {
    /// Drop the trace information from this sample
    pub fn drop_trace(self) -> TracedSample<T, S> {
        TracedSample { page: None, ..self }
    }
    /// Demote to a [TimedSample], combining the input and output values into a tuple.
    pub fn to_timed_sample(self) -> TimedSample<(T, S)> {
        TimedSample {
            time: self.time,
            value: (self.input, self.output),
            trace_status: if self.page.is_some() {
                crate::types::timed_sample::TraceStatus::Traced
            } else {
                crate::types::timed_sample::TraceStatus::Untraced
            },
        }
    }
    /// Allows you to remap the value of a [TraceSample] while keeping the
    /// time and debug page information untouched.
    pub fn map<U: Digital, V: Digital>(
        self,
        f: impl FnOnce((T, S)) -> (U, V),
    ) -> TracedSample<U, V> {
        let (input, output) = f((self.input, self.output));
        TracedSample {
            time: self.time,
            input,
            output,
            page: self.page,
        }
    }
}

/// A guard for a traced sample that ensures that its trace
/// page is active while the guard is alive.
pub struct TraceSampleGuard<T: Digital, S: Digital> {
    inner: TracedSample<T, S>,
    marker: std::marker::PhantomData<*const ()>,
}

impl<T: Digital, S: Digital> TracedSample<T, S> {
    /// Activate the trace page for this sample, returning a guard
    pub fn activate(mut self) -> TraceSampleGuard<T, S> {
        set_trace_page(self.page.take());
        TraceSampleGuard {
            inner: self,
            marker: std::marker::PhantomData,
        }
    }
}

impl<T: Digital, S: Digital> TraceSampleGuard<T, S> {
    /// Release the guard, returning the traced sample with
    /// the trace page updated to reflect any changes made
    /// while the guard was active.
    pub fn release(mut self) -> TracedSample<T, S> {
        let page = take_trace_page();
        self.inner.page = page;
        self.inner
    }
}
