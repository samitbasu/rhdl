use crate::{
    clock::clock,
    types::{clock_reset::clock_reset, reset::reset},
    Clock, ClockReset, Digital,
};

pub mod stream;
pub mod traced_simulation;
pub mod validation_simulation;
pub mod verilog_testbench;

#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub struct TimedSample<T: Digital> {
    pub value: T,
    pub time: u64,
}

pub fn timed_sample<T: Digital>(value: T, time: u64) -> TimedSample<T> {
    TimedSample { value, time }
}
