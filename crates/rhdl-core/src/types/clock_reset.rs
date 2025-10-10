use crate::{Clock, Digital, Kind, Reset, bitx::BitX};

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
                    name: "clock".to_string().into(),
                    kind: <Clock as Digital>::static_kind(),
                },
                Field {
                    name: "reset".to_string().into(),
                    kind: <Reset as Digital>::static_kind(),
                },
            ]
            .into(),
        )
    }
    fn bin(self) -> Box<[BitX]> {
        [self.clock.bin(), self.reset.bin()].concat().into()
    }
    fn dont_care() -> Self {
        Self {
            clock: Clock::dont_care(),
            reset: Reset::dont_care(),
        }
    }
}
