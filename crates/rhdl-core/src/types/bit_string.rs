//! A string of 3-value bits `0,1,x` that may be signed or unsigned.
use crate::{
    Kind, RHDLError, TypedBits,
    bitx::{BitX, bitx_string},
};

use rhdl_vlog as vlog;

/// A string of 3-value bits (0, 1, x) that may be signed or unsigned.
#[derive(Clone, PartialEq, Hash)]
pub enum BitString {
    /// A signed bit string
    Signed(Vec<BitX>),
    /// An unsigned bit string
    Unsigned(Vec<BitX>),
}

impl From<&BitString> for vlog::LitVerilog {
    fn from(value: &BitString) -> Self {
        let bits = value.bits();
        let len = bits.len();
        let sign_base = if value.is_signed() { "sb" } else { "b" };
        let s: String = match value {
            BitString::Signed(bits) => bitx_string(bits),
            BitString::Unsigned(bits) => bitx_string(bits),
        };
        vlog::lit_verilog(len as u32, &format!("{}{}", sign_base, s))
    }
}

impl From<&BitString> for Kind {
    fn from(value: &BitString) -> Self {
        match value {
            BitString::Signed(x) => Kind::Signed(x.len()),
            BitString::Unsigned(x) => Kind::Bits(x.len()),
        }
    }
}

impl From<BitString> for Kind {
    fn from(value: BitString) -> Self {
        match value {
            BitString::Signed(x) => Kind::Signed(x.len()),
            BitString::Unsigned(x) => Kind::Bits(x.len()),
        }
    }
}

impl BitString {
    /// Is this a signed bit string?
    pub fn is_signed(&self) -> bool {
        matches!(self, BitString::Signed(_))
    }
    /// Get the length of the bit string
    pub fn len(&self) -> usize {
        match self {
            BitString::Signed(bits) => bits.len(),
            BitString::Unsigned(bits) => bits.len(),
        }
    }
    /// Is this an empty bit string?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Get the bits of the bit string
    pub fn bits(&self) -> &[BitX] {
        match self {
            BitString::Signed(bits) => bits,
            BitString::Unsigned(bits) => bits,
        }
    }
    /// Cast this bit string to an unsigned bit string of the given length
    pub fn unsigned_cast(&self, len: usize) -> Result<BitString, RHDLError> {
        let tb: TypedBits = self.into();
        let bs = tb.unsigned_cast(len)?;
        Ok(bs.into())
    }
    /// Cast this bit string to a signed bit string of the given length
    pub fn signed_cast(&self, len: usize) -> Result<BitString, RHDLError> {
        let tb: TypedBits = self.into();
        let bs = tb.signed_cast(len)?;
        Ok(bs.into())
    }
    /// Resize this bit string to the given length
    pub fn resize(&self, len: usize) -> Result<BitString, RHDLError> {
        let tb: TypedBits = self.into();
        let bs = tb.resize(len)?;
        Ok(bs.into())
    }
    /// Is this bit string all zeros?
    pub fn is_zero(&self) -> bool {
        self.bits().iter().all(|b| *b == BitX::Zero)
    }
    /// Is this bit string all ones?
    pub fn is_ones(&self) -> bool {
        self.bits().iter().all(|b| *b == BitX::One)
    }
    /// Create a bit string of the given length, filled with zeros
    pub fn zeros(len: usize) -> BitString {
        BitString::Unsigned(std::iter::repeat_n(BitX::Zero, len).collect())
    }
    /// Create a bit string of the given length, filled with don't cares
    pub fn dont_care(&self) -> BitString {
        match self {
            BitString::Signed(bits) => BitString::Signed(bits.iter().map(|_| BitX::X).collect()),
            BitString::Unsigned(bits) => {
                BitString::Unsigned(bits.iter().map(|_| BitX::X).collect())
            }
        }
    }
    /// Create a bit string of the given kind, filled with don't cares
    pub fn dont_care_from_kind(kind: Kind) -> BitString {
        if kind.is_signed() {
            BitString::Signed(std::iter::repeat_n(BitX::X, kind.bits()).collect())
        } else {
            BitString::Unsigned(std::iter::repeat_n(BitX::X, kind.bits()).collect())
        }
    }
}

impl std::fmt::Display for BitString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitString::Signed(bits) => {
                write!(f, "s{}", bitx_string(bits))?;
                Ok(())
            }
            BitString::Unsigned(bits) => {
                write!(f, "b{}", bitx_string(bits))?;
                Ok(())
            }
        }
    }
}

impl std::fmt::Debug for BitString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl From<&BitString> for TypedBits {
    fn from(bs: &BitString) -> Self {
        if bs.is_signed() {
            TypedBits::new(bs.bits().to_owned(), Kind::make_signed(bs.len()))
        } else {
            TypedBits::new(bs.bits().to_owned(), Kind::make_bits(bs.len()))
        }
    }
}

impl From<BitString> for TypedBits {
    fn from(bs: BitString) -> Self {
        (&bs).into()
    }
}

impl From<&TypedBits> for BitString {
    fn from(tb: &TypedBits) -> Self {
        if tb.kind().is_signed() {
            BitString::Signed(tb.bits().to_vec())
        } else {
            BitString::Unsigned(tb.bits().to_vec())
        }
    }
}

impl From<TypedBits> for BitString {
    fn from(tb: TypedBits) -> Self {
        (&tb).into()
    }
}

impl FromIterator<BitX> for BitString {
    fn from_iter<I: IntoIterator<Item = BitX>>(iter: I) -> Self {
        let bits: Vec<BitX> = iter.into_iter().collect();
        BitString::Unsigned(bits)
    }
}

impl From<bool> for BitString {
    fn from(b: bool) -> Self {
        BitString::Unsigned(vec![b.into()])
    }
}

impl From<BitX> for BitString {
    fn from(b: BitX) -> Self {
        BitString::Unsigned(vec![b])
    }
}

impl From<&bool> for BitString {
    fn from(b: &bool) -> Self {
        BitString::Unsigned(vec![(*b).into()])
    }
}

impl From<&BitX> for BitString {
    fn from(b: &BitX) -> Self {
        BitString::Unsigned(vec![*b])
    }
}
