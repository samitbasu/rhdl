#![warn(missing_docs)]
use crate::Digital;

/// Should the given timed sample be traced?
#[derive(Copy, Clone, Debug, PartialEq, Hash, Default)]
pub enum TraceStatus {
    /// The sample should not be traced.
    Untraced,
    /// The sample should be traced.
    #[default]
    Traced,
}

impl TraceStatus {
    /// Returns true if the sample is traced.
    pub fn is_traced(&self) -> bool {
        matches!(self, TraceStatus::Traced)
    }
    /// Returns true if the sample is untraced.
    pub fn is_untraced(&self) -> bool {
        matches!(self, TraceStatus::Untraced)
    }
}

/// A sample of a digital value at a specific time.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub struct TimedSample<T: Digital> {
    /// The digital value being sampled.
    pub value: T,
    /// The time at which the sample was taken.
    pub time: u64,
    /// The trace status of the sample.
    pub trace_status: TraceStatus,
}

/// Creates a new [TimedSample] with the given time and value.
pub fn timed_sample<T: Digital>(time: u64, value: T) -> TimedSample<T> {
    TimedSample {
        value,
        time,
        trace_status: TraceStatus::default(),
    }
}

impl<T: Digital> TimedSample<T> {
    /// Allows you to remap the value of a [TimedSample] while keeping the time the same.
    pub fn map<S: Digital>(self, f: impl FnOnce(T) -> S) -> TimedSample<S> {
        TimedSample {
            value: f(self.value),
            time: self.time,
            trace_status: self.trace_status,
        }
    }
    /// Returns true if the sample is traced.
    pub fn is_traced(&self) -> bool {
        self.trace_status.is_traced()
    }
}

impl<T: Digital> std::fmt::Display for TimedSample<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}: {:?}{}",
            self.time,
            self.value.typed_bits(),
            if self.trace_status == TraceStatus::Untraced {
                "*"
            } else {
                ""
            }
        )
    }
}
