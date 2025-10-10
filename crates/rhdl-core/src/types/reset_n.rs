use crate::{Digital, Kind, bitx::BitX};

// An active Low reset signal.
#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub struct ResetN(bool);

impl ResetN {
    pub fn raw(&self) -> bool {
        self.0
    }
    pub fn any(self) -> bool {
        self.0
    }
    pub fn all(self) -> bool {
        self.0
    }
}

pub fn reset_n(b: bool) -> ResetN {
    ResetN(b)
}

impl Digital for ResetN {
    const BITS: usize = 1;
    fn static_kind() -> Kind {
        Kind::make_bool()
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rhdl_trace_type::TraceType::Reset
    }
    fn bin(self) -> Box<[BitX]> {
        [self.0.into()].into()
    }
    fn dont_care() -> Self {
        ResetN(true)
    }
}
