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

pub trait TraceContainer: Default {
    fn record<T: Digital, S: Digital>(
        &mut self,
        sample: &trace_sample::TracedSample<T, S>,
    ) -> Result<(), RHDLError>;
}
