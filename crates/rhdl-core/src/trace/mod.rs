//! Trace recording and export (as VCD or SVG) functionality
#![warn(missing_docs)]

//pub mod bit;
pub mod container;
pub mod key;
pub mod meta;
pub mod page;
pub mod record;
pub mod rtt;
pub mod session;
pub mod trace_sample;
pub mod trace_tree;
pub mod traceable;

/// A unique identifier for a traced value across all
/// pages in a simulation session.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraceId(u64);

impl nohash::IsEnabled for TraceId {}
