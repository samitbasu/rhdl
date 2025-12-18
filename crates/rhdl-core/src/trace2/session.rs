use std::sync::{Arc, RwLock};

use crate::{
    Digital, TimedSample,
    trace2::{meta::TraceMetadata, page::TracePage, trace_sample::TraceSample},
};

#[derive(Default)]
pub struct Session {
    db: Arc<RwLock<TraceMetadata>>,
}

impl Session {
    pub fn traced<T: Digital>(&self, x: TimedSample<T>) -> TraceSample<T> {
        TraceSample {
            inner: x,
            page: Some(Box::new(TracePage::new(self.db.clone()))),
        }
    }
    pub fn untraced<T: Digital>(&self, x: TimedSample<T>) -> TraceSample<T> {
        TraceSample {
            inner: x,
            page: None,
        }
    }
}
