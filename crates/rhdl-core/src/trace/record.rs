//! An individual record on a trace page
use crate::trace::{TraceId, traceable::Traceable};

/// An event reorded on a trace page.  Captures the trace
/// ID that generated this data element, along with a
/// value.
pub struct Record {
    /// The trace_id for this data (may be multiple
    /// records for a given trace_id in a single clock cycle)
    pub trace_id: TraceId,
    /// The data for this record.
    pub data: Box<dyn Traceable>,
}
