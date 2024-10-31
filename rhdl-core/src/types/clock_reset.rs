use crate::{Clock, Digital, Kind, Notable, NoteKey, NoteWriter, Reset};

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
        let mut out = vec![];
        out.extend(self.clock.bin());
        out.extend(self.reset.bin());
        out
    }
    fn init() -> Self {
        Self {
            clock: Clock::init(),
            reset: Reset::init(),
        }
    }
}

impl Notable for ClockReset {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        self.clock.note((key, "clock"), &mut writer);
        self.reset.note((key, "reset"), &mut writer);
    }
}
