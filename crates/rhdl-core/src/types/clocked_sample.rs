//! A sample that includes a value, and a [ClockReset] field along with a timestamp.

use crate::{ClockReset, Digital};

/// A sample of a digital value at a specific time, along with clock and reset information.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ClockedSample<T: Digital> {
    /// The clock and reset information.
    pub cr: ClockReset,
    /// The digital value being sampled.
    pub value: T,
    /// The time at which the sample was taken.
    pub time: u64,
}
