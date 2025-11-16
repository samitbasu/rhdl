#![warn(missing_docs)]
//! Module for BitX type and related utilities.
//!
//! This module defines the `BitX` enum representing a tri-state logic value
//! with possible values 0, 1, and X (unknown).
//! It also provides utility functions for working with vectors and strings of `BitX` values.
//!

pub mod dyn_bit_manip;

/// Represents a tri-state bit value: 0, 1, or X (unknown).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BitX {
    /// Represents a logical 0.
    Zero,
    /// Represents a logical 1.
    One,
    /// Represents an unknown value
    X,
}

/// Create a boxed slice of `BitX` from a slice of booleans.
pub fn bitx_vec(x: &[bool]) -> Box<[BitX]> {
    x.iter().map(|&b| b.into()).collect()
}

/// Create a printable string from a slice of `BitX`.
pub fn bitx_string(x: &[BitX]) -> String {
    x.iter().rev().map(|b| char::from(*b)).collect()
}

/// Parse a string into a boxed slice of `BitX`. Returns `None` if the string
/// contains invalid characters.
pub fn bitx_parse(x: &str) -> Option<Box<[BitX]>> {
    x.chars()
        .map(|c| match c {
            '0' => Some(BitX::Zero),
            '1' => Some(BitX::One),
            'x' | 'X' => Some(BitX::X),
            _ => None,
        })
        .rev()
        .collect()
}

impl BitX {
    /// Convert the `BitX` value to an `Option<bool>`.
    /// Returns `Some(true)` for `One`, `Some(false)` for `Zero`, and `None` for `X`.
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
