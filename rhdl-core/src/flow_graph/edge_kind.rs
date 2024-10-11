use crate::types::bit_string::BitString;

#[derive(Clone, Hash)]
pub enum EdgeKind {
    ArgBit(usize, usize),
    Clock,
    Reset,
    Selector(usize),
    Splice(usize),
    True,
    False,
    DynamicOffset(usize),
    Virtual,
}

impl std::fmt::Debug for EdgeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArgBit(arg, bit) => {
                write!(f, "a{}[{}]", arg, bit)
            }
            Self::Selector(ndx) => write!(f, "sel[{ndx}]"),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::DynamicOffset(bit) => write!(f, "dyn[{}]", bit),
            Self::Virtual => write!(f, "virt"),
            Self::Splice(bit) => write!(f, "splice[{}]", bit),
            Self::Clock => write!(f, "clk"),
            Self::Reset => write!(f, "rst"),
        }
    }
}
