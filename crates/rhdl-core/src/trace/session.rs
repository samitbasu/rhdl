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
pub struct Session {
    db: Arc<RwLock<TraceMetadata>>,
}

impl Session {
    fn traced<T: Digital, S: Digital>(&self, x: TimedSample<T>, output: S) -> TracedSample<T, S> {
        TracedSample {
            output,
            page: Some(self.page()),
            time: x.time,
            input: x.value,
        }
    }
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
    pub fn page(&self) -> Box<TracePage> {
        Box::new(TracePage::new(self.db.clone()))
    }
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
