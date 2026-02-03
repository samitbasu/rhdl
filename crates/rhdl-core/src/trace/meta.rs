//! Metadata about a traced value
use rhdl_trace_type::RTT;

use crate::{Kind, trace::trace_tree::TraceTree};

/// Details about a traced value.
pub struct TraceDetails {
    /// The Trace ID for this time series (useful for reverse lookups)
    pub trace_id: super::TraceId,
    /// The hierarchical path to the recorded value.
    pub path: Vec<&'static str>,
    /// The name of the recorded value.
    pub key: String,
    /// The number of trace bits allocated to this value
    pub width: usize,
    /// The [Kind](crate::Kind) of this value.
    pub kind: Kind,
}

/// The trace details stores an ID for each trace location.  
/// As these are accessed many many times, we store them in a
/// structure that can be shared between pages and ultimately
/// between threads.
#[derive(Default)]
pub struct TraceMetadata {
    details: nohash::IntMap<super::TraceId, TraceDetails>,
}

impl TraceMetadata {
    /// Check if the given trace ID exists in the database.
    pub fn has_key(&self, id: super::TraceId) -> bool {
        self.details.contains_key(&id)
    }
    /// Get the details for a given trace ID.
    pub fn get_details(&self, id: super::TraceId) -> Option<&TraceDetails> {
        self.details.get(&id)
    }
    /// Insert new trace details into the database.
    pub fn insert(&mut self, id: super::TraceId, details: TraceDetails) {
        self.details.insert(id, details);
    }
    /// Build a [TraceTree] representation of the trace metadata.
    pub(crate) fn build_trace_tree(&self) -> TraceTree {
        TraceTree::build(self.details.values())
    }
    pub(crate) fn rtt(&self) -> RTT {
        RTT::TraceInfo(
            self.details
                .values()
                .map(|details| {
                    let name = format!(
                        "{}.{}",
                        [&["top"], &details.path[..]].concat().join("."),
                        details.key
                    );
                    (name, details.kind.into())
                })
                .collect(),
        )
    }
}
