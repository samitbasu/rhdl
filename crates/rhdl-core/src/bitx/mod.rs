pub mod dyn_bit_manip;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BitX {
    Zero,
    One,
    X,
}

pub fn bitx_vec(x: &[bool]) -> Box<[BitX]> {
    x.iter().map(|&b| b.into()).collect()
}

pub fn bitx_string(x: &[BitX]) -> String {
    x.iter().rev().map(|b| char::from(*b)).collect()
}

pub fn bitx_parse(x: &str) -> Option<Box<[BitX]>> {
    x.chars()
        .map(|c| match c {
            '0' => Some(BitX::Zero),
            '1' => Some(BitX::One),
            'x' => Some(BitX::X),
            _ => None,
        })
        .rev()
        .collect()
}

impl BitX {
    pub fn to_bool(self) -> Option<bool> {
        match self {
            BitX::Zero => Some(false),
            BitX::One => Some(true),
            BitX::X => None,
        }
    }
}

impl From<BitX> for char {
    fn from(b: BitX) -> Self {
        match b {
            BitX::Zero => '0',
            BitX::One => '1',
            BitX::X => 'x',
        }
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

impl From<bool> for BitX {
    fn from(val: bool) -> Self {
        if val { BitX::One } else { BitX::Zero }
    }
}

impl std::ops::BitAndAssign for BitX {
    fn bitand_assign(&mut self, rhs: BitX) {
        *self = *self & rhs;
    }
}

impl std::ops::BitAnd for BitX {
    type Output = BitX;

    fn bitand(self, rhs: BitX) -> BitX {
        match (self, rhs) {
            (BitX::Zero, _) | (_, BitX::Zero) => BitX::Zero,
            (BitX::One, BitX::One) => BitX::One,
            _ => BitX::X,
        }
    }
}

impl std::ops::BitOrAssign for BitX {
    fn bitor_assign(&mut self, rhs: BitX) {
        *self = *self | rhs;
    }
}

impl std::ops::BitOr for BitX {
    type Output = BitX;

    fn bitor(self, rhs: BitX) -> BitX {
        match (self, rhs) {
            (BitX::One, _) | (_, BitX::One) => BitX::One,
            (BitX::Zero, BitX::Zero) => BitX::Zero,
            _ => BitX::X,
        }
    }
}

impl std::ops::BitXor for BitX {
    type Output = BitX;

    fn bitxor(self, rhs: BitX) -> BitX {
        match (self, rhs) {
            (BitX::One, BitX::Zero) | (BitX::Zero, BitX::One) => BitX::One,
            (BitX::Zero, BitX::Zero) | (BitX::One, BitX::One) => BitX::Zero,
            _ => BitX::X,
        }
    }
}

impl std::ops::Neg for BitX {
    type Output = BitX;

    fn neg(self) -> BitX {
        match self {
            BitX::Zero => BitX::One,
            BitX::One => BitX::Zero,
            BitX::X => BitX::X,
        }
    }
}

impl std::ops::Not for BitX {
    type Output = BitX;

    fn not(self) -> BitX {
        match self {
            BitX::Zero => BitX::One,
            BitX::One => BitX::Zero,
            BitX::X => BitX::X,
        }
    }
}
