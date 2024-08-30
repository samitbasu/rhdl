use crate::types::bit_string::BitString;

#[derive(Clone)]
pub enum EdgeKind {
    Arg(usize),
    ArgBit(usize, usize),
    Selector(usize),
    OutputBit(usize),
    Splice(usize),
    True,
    False,
    DynamicOffset(usize),
    CaseLiteral(BitString),
    CaseWild,
    Virtual,
}

impl std::fmt::Debug for EdgeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Arg(arg0) => {
                if *arg0 != 0 {
                    write!(f, "a{}", arg0)
                } else {
                    Ok(())
                }
            }
            Self::ArgBit(arg, bit) => {
                write!(f, "a{}[{}]", arg, bit)
            }
            Self::Selector(ndx) => write!(f, "sel[{ndx}]"),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::DynamicOffset(bit) => write!(f, "dyn[{}]", bit),
            Self::CaseLiteral(arg0) => write!(f, "{:?}", arg0),
            Self::CaseWild => write!(f, "_"),
            Self::Virtual => write!(f, "virt"),
            Self::OutputBit(bit) => write!(f, "o[{}]", bit),
            Self::Splice(bit) => write!(f, "splice[{}]", bit),
        }
    }
}
