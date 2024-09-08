use crate::{util::binary_string, Kind, RHDLError, TypedBits};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum BitString {
    Signed(Vec<bool>),
    Unsigned(Vec<bool>),
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
    pub fn bits(&self) -> &[bool] {
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
        self.bits().iter().filter(|b| **b).count()
    }
    pub fn trailing_zeros(&self) -> usize {
        self.bits().iter().take_while(|b| !*b).count()
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.bits().iter().all(|b| !*b)
    }
}

impl std::fmt::Debug for BitString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitString::Signed(bits) => {
                write!(f, "s{}", binary_string(bits))?;
                Ok(())
            }
            BitString::Unsigned(bits) => {
                write!(f, "b{}", binary_string(bits))?;
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
