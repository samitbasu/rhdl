use crate::{bitx::BitX, Clock, Digital, Kind, Reset};

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
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rhdl_trace_type::TraceType::Struct(rhdl_trace_type::Struct {
            name: "ClockReset".into(),
            fields: vec![
                rhdl_trace_type::Field {
                    name: "clock".into(),
                    ty: <Clock as Digital>::static_trace_type(),
                },
                rhdl_trace_type::Field {
                    name: "reset".into(),
                    ty: <Reset as Digital>::static_trace_type(),
                },
            ],
        })
    }
    fn bin(self) -> Vec<BitX> {
        [self.clock.bin().as_slice(), self.reset.bin().as_slice()].concat()
    }
    fn dont_care() -> Self {
        Self {
            clock: Clock::dont_care(),
            reset: Reset::dont_care(),
        }
    }
}
