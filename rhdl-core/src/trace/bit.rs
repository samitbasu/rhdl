#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TraceBit {
    Zero,
    One,
    X,
    Z,
}

impl From<bool> for TraceBit {
    fn from(b: bool) -> Self {
        if b {
            TraceBit::One
        } else {
            TraceBit::Zero
        }
    }
}

pub enum TraceValue {
    TwoValued(Vec<bool>),
    FourValued(Vec<TraceBit>),
}
