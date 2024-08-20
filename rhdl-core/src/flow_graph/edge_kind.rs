use crate::rtl::object::BitString;

#[derive(Clone)]
pub enum EdgeKind {
    Arg(usize),
    Selector,
    True,
    False,
    DynamicOffset,
    Splice,
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
            Self::Selector => write!(f, "sel"),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::DynamicOffset => write!(f, "dyn_offset"),
            Self::Splice => write!(f, "splice"),
            Self::CaseLiteral(arg0) => write!(f, "{:?}", arg0),
            Self::CaseWild => write!(f, "_"),
            Self::Virtual => write!(f, "virt"),
        }
    }
}
