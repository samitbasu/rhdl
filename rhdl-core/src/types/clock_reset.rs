use crate::{Clock, Digital, Kind, Reset};

use super::kind::Field;

#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub struct ClockReset {
    pub clock: Clock,
    pub reset: Reset,
}

pub fn clock_reset(clock: Clock, reset: Reset) -> ClockReset {
    ClockReset { clock, reset }
}

impl Digital for ClockReset {
    const BITS: usize = Clock::BITS + Reset::BITS;
    fn static_kind() -> Kind {
        Kind::make_struct(
            "ClockReset",
            vec![
                Field {
                    name: "clock".into(),
                    kind: <Clock as Digital>::static_kind(),
                },
                Field {
                    name: "reset".into(),
                    kind: <Reset as Digital>::static_kind(),
                },
            ],
        )
    }
    fn bin(self) -> Vec<bool> {
        [self.clock.bin().as_slice(), self.reset.bin().as_slice()].concat()
    }
    fn init() -> Self {
        Self {
            clock: Clock::init(),
            reset: Reset::init(),
        }
    }
}
