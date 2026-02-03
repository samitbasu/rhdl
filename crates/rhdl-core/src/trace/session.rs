//! A trace session
use std::sync::{Arc, RwLock};

use crate::{
    Digital, TimedSample,
    trace::{
        meta::TraceMetadata,
        page::{TracePage, set_trace_page, take_trace_page},
        trace_sample::TracedSample,
    },
};

#[derive(Default, Clone)]
/// A trace session is the logical container for metadata
/// related to a set of traces.  You can think of it like a
/// VCD file session, or a waveform viewer session.
pub struct Session {
    db: Arc<RwLock<TraceMetadata>>,
}

impl Session {
    /// Add a timed input sample with the associated output to the
    /// trace session.  Returns the traced sample.
    pub fn traced<T: Digital, S: Digital>(
        &self,
        x: TimedSample<T>,
        output: S,
    ) -> TracedSample<T, S> {
        TracedSample {
            output,
            page: Some(self.page()),
            time: x.time,
            input: x.value,
        }
    }
    /// Convenient method to create an untraced sample.
    pub fn untraced<T: Digital, S: Digital>(
        &self,
        x: TimedSample<T>,
        output: S,
    ) -> TracedSample<T, S> {
        TracedSample {
            output,
            page: None,
            time: x.time,
            input: x.value,
        }
    }
    /// Transform a timed sample and output into a traced sample,
    /// depending on whether the input is flagged as traced or not.
    pub fn transform<T: Digital, S: Digital>(
        &self,
        x: TimedSample<T>,
        output: S,
    ) -> TracedSample<T, S> {
        if x.trace_status.is_traced() {
            self.traced(x, output)
        } else {
            self.untraced(x, output)
        }
    }
    pub(crate) fn page(&self) -> Box<TracePage> {
        Box::new(TracePage::new(self.db.clone()))
    }
    /// Helper utility.  Run the supplied function with a trace page active at time `time`.
    pub fn traced_at_time(&self, time: u64, func: impl FnOnce()) -> TracedSample<(), ()> {
        set_trace_page(Some(self.page()));
        func();
        TracedSample {
            time,
            input: (),
            output: (),
            page: take_trace_page(),
        }
    }
}
