use crate::{
    clock::clock,
    types::{clock_reset::clock_reset, reset::reset},
    Clock, ClockReset, Digital,
};

pub mod traced_simulation;
pub mod verilog_testbench;

pub struct TimedSample<T: Digital> {
    pub value: T,
    pub time: u64,
}

pub fn timed_sample<T: Digital>(value: T, time: u64) -> TimedSample<T> {
    TimedSample { value, time }
}

pub fn sim_clock(period: u64) -> impl Iterator<Item = TimedSample<Clock>> {
    (0..).map(move |phase| TimedSample {
        value: clock(phase % 2 == 0),
        time: phase * period,
    })
}

pub fn sim_clock_reset(
    mut clock: impl Iterator<Item = TimedSample<Clock>>,
) -> impl Iterator<Item = TimedSample<ClockReset>> {
    let mut clock_count = 0;
    std::iter::from_fn(move || {
        if let Some(sample) = clock.next() {
            clock_count += 1;
            if clock_count < 4 {
                Some(timed_sample(
                    clock_reset(sample.value, reset(true)),
                    sample.time,
                ))
            } else {
                Some(timed_sample(
                    clock_reset(sample.value, reset(false)),
                    sample.time,
                ))
            }
        } else {
            None
        }
    })
}
