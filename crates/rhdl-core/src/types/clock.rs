use crate::{Digital, Kind, bitx::BitX};

#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub struct Clock(bool);

impl Clock {
    pub fn raw(&self) -> bool {
        self.0
    }
}

pub fn clock(b: bool) -> Clock {
    Clock(b)
}

impl Digital for Clock {
    const BITS: usize = 1;
    fn static_kind() -> Kind {
        Kind::make_bool()
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rhdl_trace_type::TraceType::Clock
    }
    fn bin(self) -> Box<[BitX]> {
        [self.0.into()].into()
    }
    fn dont_care() -> Self {
        Clock(false)
    }
}
