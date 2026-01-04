//! Trace recording and export (as VCD or SVG) functionality
#![warn(missing_docs)]
use crate::{Digital, RHDLError};

pub mod bit;
pub mod key;
pub mod meta;
pub mod page;
pub mod record;
pub mod rtt;
pub mod session;
pub mod svg;
pub mod trace_sample;
pub mod trace_tree;
pub mod traceable;
pub mod vcd;

/// A unique identifier for a traced value across all
/// pages in a simulation session.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraceId(u64);

impl nohash::IsEnabled for TraceId {}

/// A trace container is responsible for collecting
/// trace samples and exporting them to a specific format,
/// such as VCD or SVG.
pub trait TraceContainer: Default {
    /// Record a traced sample into the trace container.
    fn record<T: Digital, S: Digital>(
        &mut self,
        sample: &trace_sample::TracedSample<T, S>,
    ) -> Result<(), RHDLError>;
}
