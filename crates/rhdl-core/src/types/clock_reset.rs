//! A combined clock and reset type
use crate::{Clock, Digital, Kind, Reset, bitx::BitX};

use super::kind::Field;

/// A combined clock and reset type.
///
/// Clock and Reset signals are usually passed around together in hardware designs.
/// This type encapsulates both signals into a single struct.
#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub struct ClockReset {
    /// The clock signal.
    pub clock: Clock,
    /// The reset signal.
    pub reset: Reset,
}

impl std::fmt::Display for ClockReset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "c={},r={}",
            u8::from(self.clock.raw()),
            u8::from(self.reset.raw())
        )
    }
}

/// Create a combined clock and reset from individual signals.
/// This is not a synthesizable function.  It's for testing.
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
