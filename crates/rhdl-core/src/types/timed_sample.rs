#![warn(missing_docs)]
use crate::Digital;

/// A sample of a digital value at a specific time.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub struct TimedSample<T: Digital> {
    /// The digital value being sampled.
    pub value: T,
    /// The time at which the sample was taken.
    pub time: u64,
}

/// Creates a new [TimedSample] with the given time and value.
pub fn timed_sample<T: Digital>(time: u64, value: T) -> TimedSample<T> {
    TimedSample { value, time }
}

impl<T: Digital> TimedSample<T> {
    /// Allows you to remap the value of a [TimedSample] while keeping the time the same.
    pub fn map<S: Digital>(self, f: impl FnOnce(T) -> S) -> TimedSample<S> {
        TimedSample {
            value: f(self.value),
            time: self.time,
        }
    }
}

impl<T: Digital> std::fmt::Display for TimedSample<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.time, self.value.typed_bits())
    }
}
