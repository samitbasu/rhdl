use rhdl::{core::clock, prelude::*};

#[derive(PartialEq, Clone, Copy, Debug, Digital)]
pub struct ClockReset {
    pub clock: Clock,
    pub reset: Reset,
}

pub fn clock_reset(clock: Clock, reset: Reset) -> ClockReset {
    ClockReset { clock, reset }
}
