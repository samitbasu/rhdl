use crate::rhdl_core::{
    Kind, RHDLError, TypedBits,
    bitx::{BitX, bitx_string},
};

#[derive(Clone, PartialEq, Hash)]
pub enum BitString {
    Signed(Vec<BitX>),
    Unsigned(Vec<BitX>),
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
    pub fn signed(bits: Vec<BitX>) -> BitString {
        BitString::Signed(bits)
    }
    pub fn unsigned(bits: Vec<BitX>) -> BitString {
        BitString::Unsigned(bits)
    }
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
        self.bits().iter().filter(|b| **b == BitX::One).count()
    }
    pub fn trailing_zeros(&self) -> usize {
        self.bits().iter().take_while(|b| **b == BitX::Zero).count()
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.bits().iter().all(|b| *b == BitX::Zero)
    }

    pub(crate) fn is_ones(&self) -> bool {
        self.bits().iter().all(|b| *b == BitX::One)
    }

    pub(crate) fn zeros(shift_amount: usize) -> BitString {
        BitString::Unsigned(std::iter::repeat_n(BitX::Zero, shift_amount).collect())
    }

    pub(crate) fn dont_care(&self) -> BitString {
        match self {
            BitString::Signed(bits) => BitString::Signed(bits.iter().map(|_| BitX::X).collect()),
            BitString::Unsigned(bits) => {
                BitString::Unsigned(bits.iter().map(|_| BitX::X).collect())
            }
        }
    }
    pub(crate) fn dont_care_from_kind(kind: Kind) -> BitString {
        if kind.is_signed() {
            BitString::Signed(std::iter::repeat_n(BitX::X, kind.bits()).collect())
        } else {
            BitString::Unsigned(std::iter::repeat_n(BitX::X, kind.bits()).collect())
        }
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
