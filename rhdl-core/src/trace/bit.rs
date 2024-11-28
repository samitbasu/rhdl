use crate::bitx::BitX;

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

impl From<BitX> for TraceBit {
    fn from(b: BitX) -> Self {
        match b {
            BitX::Zero => TraceBit::Zero,
            BitX::One => TraceBit::One,
            BitX::X => TraceBit::X,
        }
    }
}
