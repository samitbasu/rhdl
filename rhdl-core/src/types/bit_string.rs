use std::iter::repeat;

use crate::{Kind, RHDLError, TypedBits};

use super::bitx::{bitx_string, BitX};

#[derive(Clone, PartialEq, Hash)]
pub enum BitString {
    Signed(Vec<BitX>),
    Unsigned(Vec<BitX>),
}

impl BitString {
    pub fn is_signed(&self) -> bool {
        matches!(self, BitString::Signed(_))
    }
    pub fn is_unsigned(&self) -> bool {
        matches!(self, BitString::Unsigned(_))
    }
    pub fn len(&self) -> usize {
        match self {
            BitString::Signed(bits) => bits.len(),
            BitString::Unsigned(bits) => bits.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn bits(&self) -> &[BitX] {
        match self {
            BitString::Signed(bits) => bits,
            BitString::Unsigned(bits) => bits,
        }
    }
    pub fn unsigned_cast(&self, len: usize) -> Result<BitString, RHDLError> {
        let tb: TypedBits = self.into();
        let bs = tb.unsigned_cast(len)?;
        Ok(bs.into())
    }
    pub fn signed_cast(&self, len: usize) -> Result<BitString, RHDLError> {
        let tb: TypedBits = self.into();
        let bs = tb.signed_cast(len)?;
        Ok(bs.into())
    }
    pub fn resize(&self, len: usize) -> Result<BitString, RHDLError> {
        let tb: TypedBits = self.into();
        let bs = tb.resize(len)?;
        Ok(bs.into())
    }
    pub fn num_ones(&self) -> usize {
        self.bits().iter().filter(|&b| b.is_one()).count()
    }
    pub fn trailing_zeros(&self) -> usize {
        self.bits().iter().take_while(|b| b.is_zero()).count()
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.bits().iter().all(|b| b.is_zero())
    }

    pub(crate) fn zeros(shift_amount: usize) -> BitString {
        BitString::Unsigned(repeat(BitX::Zero).take(shift_amount).collect())
    }
}

impl std::fmt::Debug for BitString {
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

impl From<&BitString> for TypedBits {
    fn from(bs: &BitString) -> Self {
        if bs.is_signed() {
            {
                TypedBits {
                    bits: bs.bits().to_owned(),
                    kind: Kind::make_signed(bs.len()),
                }
            }
        } else {
            {
                TypedBits {
                    bits: bs.bits().to_owned(),
                    kind: Kind::make_bits(bs.len()),
                }
            }
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
        if tb.kind.is_signed() {
            BitString::Signed(tb.bits.clone())
        } else {
            BitString::Unsigned(tb.bits.clone())
        }
    }
}

impl From<TypedBits> for BitString {
    fn from(tb: TypedBits) -> Self {
        (&tb).into()
    }
}
