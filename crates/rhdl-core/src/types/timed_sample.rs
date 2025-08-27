use crate::Digital;

#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub struct TimedSample<T: Digital> {
    pub value: T,
    pub time: u64,
}

pub fn timed_sample<T: Digital>(time: u64, value: T) -> TimedSample<T> {
    TimedSample { value, time }
}

impl<T: Digital> TimedSample<T> {
    pub fn map<S: Digital>(self, f: impl FnOnce(T) -> S) -> TimedSample<S> {
        TimedSample {
            value: f(self.value),
            time: self.time,
        }
    }
}
