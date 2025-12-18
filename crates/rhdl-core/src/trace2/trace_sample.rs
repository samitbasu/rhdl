//! A timed sample with an (option) trace page attached.
//!
//! This is how you collect trace data when using the iterator based simulation API.
use crate::{
    Digital, TimedSample,
    trace2::page::{TracePage, set_trace_page, take_trace_page},
};

/// A timed sample with an optional trace page attached.
pub struct TraceSample<T: Digital> {
    /// The timed sample.
    pub inner: TimedSample<T>,
    /// The trace page this sample belongs to.
    pub page: Option<Box<TracePage>>,
}

/// A guard for a traced sample that ensures that its trace
/// page is active while the guard is alive.
pub struct TraceSampleGuard<T: Digital> {
    inner: TraceSample<T>,
}

impl<T: Digital> TraceSample<T> {
    pub fn guard(mut self) -> TraceSampleGuard<T> {
        set_trace_page(self.page.take());
        TraceSampleGuard { inner: self }
    }
}

impl<T: Digital> TraceSampleGuard<T> {
    pub fn release(mut self) -> TraceSample<T> {
        let page = take_trace_page();
        self.inner.page = page;
        self.inner
    }
}
