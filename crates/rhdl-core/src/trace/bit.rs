//! A 4-valued bit used in RHDL traces
use crate::bitx::BitX;

/// A 4-valued bit used in RHDL traces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TraceBit {
    /// Logical zero
    Zero,
    /// Logical one
    One,
    /// Unknown value
    X,
    /// High impedance
    Z,
}

impl From<bool> for TraceBit {
    fn from(b: bool) -> Self {
        if b { TraceBit::One } else { TraceBit::Zero }
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
