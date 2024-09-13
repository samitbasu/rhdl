use crate::error::rhdl_error;
use crate::RHDLError;

use super::error::DynamicTypeError;

#[derive(Clone, Copy, Default)]
pub enum BitX {
    Zero,
    One,
    #[default]
    X,
}

impl std::hash::Hash for BitX {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            BitX::Zero => 0.hash(state),
            BitX::One => 1.hash(state),
            BitX::X => 2.hash(state),
        }
    }
}

impl std::fmt::Debug for BitX {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitX::Zero => write!(f, "0"),
            BitX::One => write!(f, "1"),
            BitX::X => write!(f, "x"),
        }
    }
}

impl std::cmp::PartialEq for BitX {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (BitX::Zero, BitX::Zero) | (BitX::One, BitX::One)
        )
    }
}

impl BitX {
    pub fn is_init(&self) -> bool {
        matches!(self, BitX::Zero | BitX::One)
    }
    pub fn is_one(&self) -> bool {
        matches!(self, BitX::One)
    }
    pub fn maybe_one(&self) -> bool {
        matches!(self, BitX::One | BitX::X)
    }

    pub fn is_zero(&self) -> bool {
        matches!(self, BitX::Zero)
    }
}

impl From<bool> for BitX {
    fn from(b: bool) -> Self {
        if b {
            BitX::One
        } else {
            BitX::Zero
        }
    }
}

impl TryInto<bool> for BitX {
    type Error = RHDLError;

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            BitX::Zero => Ok(false),
            BitX::One => Ok(true),
            BitX::X => Err(rhdl_error(DynamicTypeError::CannotConvertXToBool)),
        }
    }
}

impl std::ops::Not for BitX {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            BitX::Zero => BitX::One,
            BitX::One => BitX::Zero,
            BitX::X => BitX::X,
        }
    }
}

impl std::ops::BitAnd for BitX {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (BitX::Zero, _) | (_, BitX::Zero) => BitX::Zero,
            (BitX::One, BitX::One) => BitX::One,
            _ => BitX::X,
        }
    }
}

impl std::ops::BitOr for BitX {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (BitX::One, _) | (_, BitX::One) => BitX::One,
            (BitX::Zero, BitX::Zero) => BitX::Zero,
            _ => BitX::X,
        }
    }
}

impl std::ops::BitXor for BitX {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (BitX::Zero, BitX::One) | (BitX::One, BitX::Zero) => BitX::One,
            (BitX::Zero, BitX::Zero) | (BitX::One, BitX::One) => BitX::Zero,
            _ => BitX::X,
        }
    }
}

impl std::ops::BitAndAssign for BitX {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl std::ops::BitOrAssign for BitX {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl std::ops::BitXorAssign for BitX {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl std::fmt::Display for BitX {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitX::Zero => write!(f, "0"),
            BitX::One => write!(f, "1"),
            BitX::X => write!(f, "x"),
        }
    }
}

pub fn bitx_string(x: &[BitX]) -> String {
    x.iter().map(|x| format!("{}", x)).collect()
}
