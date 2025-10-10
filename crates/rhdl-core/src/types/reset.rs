use crate::{Digital, Kind, bitx::BitX};

#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub struct Reset(bool);

impl Reset {
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

pub fn reset(b: bool) -> Reset {
    Reset(b)
}

impl Digital for Reset {
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
        Reset(false)
    }
}
