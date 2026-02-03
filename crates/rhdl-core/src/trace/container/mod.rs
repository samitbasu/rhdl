//! Containers for trace samples
#![warn(missing_docs)]
use crate::{Digital, RHDLError, trace::trace_sample};

pub mod svg;
pub mod vcd;

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
