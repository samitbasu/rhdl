use anyhow::bail;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::iter::repeat;

use crate::{
    digital::binary_string,
    path::{bit_range, Path},
    Kind,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypedBits {
    pub bits: Vec<bool>,
    pub kind: Kind,
}

impl TypedBits {
    pub const EMPTY: TypedBits = TypedBits {
        bits: Vec::new(),
        kind: Kind::Empty,
    };

    pub fn path(&self, path: &Path) -> anyhow::Result<TypedBits> {
        let (range, kind) = bit_range(self.kind.clone(), path)?;
        Ok(TypedBits {
            bits: self.bits[range].to_vec(),
            kind,
        })
    }
    pub fn discriminant(&self) -> anyhow::Result<TypedBits> {
        if self.kind.is_enum() {
            self.path(&Path::default().discriminant())
        } else {
            Ok(self.clone())
        }
    }
    pub fn unsigned_cast(&self, bits: usize) -> anyhow::Result<TypedBits> {
        if bits > self.kind.bits() {
            return Ok(TypedBits {
                bits: self
                    .bits
                    .clone()
                    .into_iter()
                    .chain(repeat(false))
                    .take(bits)
                    .collect(),
                kind: Kind::make_bits(bits),
            });
        }
        let (base, rest) = self.bits.split_at(bits);
        if rest.iter().any(|b| *b) {
            anyhow::bail!(
                "Unsigned cast failed: {} is not representable in {} bits",
                self,
                bits
            );
        }
        Ok(TypedBits {
            bits: base.to_vec(),
            kind: Kind::make_bits(bits),
        })
    }
    pub fn signed_cast(&self, bits: usize) -> anyhow::Result<TypedBits> {
        if bits > self.kind.bits() {
            let sign_bit = self.bits.last().cloned().unwrap_or_default();
            return Ok(TypedBits {
                bits: self
                    .bits
                    .clone()
                    .into_iter()
                    .chain(repeat(sign_bit))
                    .take(bits)
                    .collect(),
                kind: Kind::make_signed(bits),
            });
        }
        let (base, rest) = self.bits.split_at(bits);
        let new_sign_bit = base.last().cloned().unwrap_or_default();
        if rest.iter().any(|b| *b != new_sign_bit) {
            anyhow::bail!(
                "Signed cast failed: {} is not representable in {} bits",
                self,
                bits
            );
        }
        Ok(TypedBits {
            bits: base.to_vec(),
            kind: Kind::make_signed(bits),
        })
    }
    pub fn as_i64(&self) -> anyhow::Result<i64> {
        let tb64 = match &self.kind {
            Kind::Bits(_) => self.unsigned_cast(64)?,
            Kind::Signed(_) => self.signed_cast(64)?,
            _ => {
                bail!("Cannot cast {:?} to i64", self.kind)
            }
        };
        let mut ret: u64 = 0;
        for ndx in 0..64 {
            ret |= (tb64.bits[ndx] as u64) << ndx;
        }
        Ok(ret as i64)
    }
}

impl std::ops::Add<TypedBits> for TypedBits {
    type Output = Result<TypedBits>;

    fn add(self, rhs: TypedBits) -> Self::Output {
        if self.kind != rhs.kind {
            bail!(
                "Cannot add {} and {} because they have different types",
                self,
                rhs
            );
        }
        let mut ret = Vec::new();
        let mut carry = false;
        for (a, b) in self.bits.iter().zip(rhs.bits.iter()) {
            let sum = a ^ b ^ carry;
            let new_carry = (a & b) | (a & carry) | (b & carry);
            ret.push(sum);
            carry = new_carry;
        }
        Ok(TypedBits {
            bits: ret,
            kind: self.kind,
        })
    }
}

impl std::fmt::Display for TypedBits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}b{:?}", binary_string(&self.bits), self.kind)
    }
}

#[cfg(test)]
mod tests {
    use crate::Digital;

    use super::*;

    #[test]
    fn test_typed_bits_add() {
        let a = 42_u8.typed_bits();
        let b = 196_u8.typed_bits();
        let c = (a + b).unwrap();
        assert_eq!(c, 238_u8.typed_bits());
    }
}
