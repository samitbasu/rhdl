use crate::Digital;

#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub struct TimedSample<T: Digital> {
    pub value: T,
    pub time: u64,
}

pub fn timed_sample<T: Digital>(value: T, time: u64) -> TimedSample<T> {
    TimedSample { value, time }
}
