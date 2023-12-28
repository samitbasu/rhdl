use anyhow::bail;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::iter::repeat;

use crate::dyn_bit_manip::bits_shr_signed;
use crate::dyn_bit_manip::{
    bit_neg, bit_not, bits_and, bits_or, bits_shl, bits_shr, bits_xor, full_add, full_sub,
};
use crate::Digital;
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
    pub fn update(&self, path: &Path, value: TypedBits) -> anyhow::Result<TypedBits> {
        let (range, kind) = bit_range(self.kind.clone(), path)?;
        if kind != value.kind {
            bail!(
                "Cannot update {} with {} because they have different types",
                self,
                value
            );
        }
        let mut new_bits = self.bits.clone();
        new_bits.splice(range, value.bits.iter().cloned());
        Ok(TypedBits {
            bits: new_bits,
            kind: self.kind.clone(),
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
    pub fn any(&self) -> TypedBits {
        self.bits.iter().any(|b| *b).typed_bits()
    }
    pub fn all(&self) -> TypedBits {
        self.bits.iter().all(|b| *b).typed_bits()
    }
    pub fn as_signed(&self) -> Result<TypedBits> {
        if let Kind::Bits(ndx) = self.kind {
            Ok(TypedBits {
                bits: self.bits.clone(),
                kind: Kind::Signed(ndx),
            })
        } else {
            bail!("Cannot cast {:?} to signed", self.kind)
        }
    }
    pub fn as_unsigned(&self) -> Result<TypedBits> {
        if let Kind::Signed(ndx) = self.kind {
            Ok(TypedBits {
                bits: self.bits.clone(),
                kind: Kind::Bits(ndx),
            })
        } else {
            bail!("Cannot cast {:?} to unsigned", self.kind)
        }
    }
    pub fn xor(&self) -> TypedBits {
        self.bits.iter().fold(false, |a, b| a ^ b).typed_bits()
    }
    pub fn as_bool(&self) -> Result<bool> {
        if self.kind.is_bool() {
            Ok(self.bits[0])
        } else {
            bail!("Cannot cast {:?} to bool", self.kind)
        }
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
        Ok(TypedBits {
            bits: full_add(&self.bits, &rhs.bits),
            kind: self.kind,
        })
    }
}

impl std::ops::Sub<TypedBits> for TypedBits {
    type Output = Result<TypedBits>;

    fn sub(self, rhs: TypedBits) -> Self::Output {
        if self.kind != rhs.kind {
            bail!(
                "Cannot subtract {} and {} because they have different types",
                self,
                rhs
            );
        }
        Ok(TypedBits {
            bits: full_sub(&self.bits, &rhs.bits),
            kind: self.kind,
        })
    }
}

impl std::ops::Not for TypedBits {
    type Output = Result<TypedBits>;

    fn not(self) -> Self::Output {
        if self.kind.is_composite() {
            bail!("Cannot negate composite {}", self);
        }
        Ok(TypedBits {
            bits: bit_not(&self.bits),
            kind: self.kind,
        })
    }
}

impl std::ops::BitXor for TypedBits {
    type Output = Result<TypedBits>;

    fn bitxor(self, rhs: TypedBits) -> Self::Output {
        if self.kind != rhs.kind {
            bail!(
                "Cannot xor {} and {} because they have different types",
                self,
                rhs
            );
        }
        if self.kind.is_composite() {
            bail!("Cannot xor composite {}", self);
        }
        Ok(TypedBits {
            bits: bits_xor(&self.bits, &rhs.bits),
            kind: self.kind,
        })
    }
}

impl std::ops::BitAnd for TypedBits {
    type Output = Result<TypedBits>;

    fn bitand(self, rhs: TypedBits) -> Self::Output {
        if self.kind != rhs.kind {
            bail!(
                "Cannot and {} and {} because they have different types",
                self,
                rhs
            );
        }
        if self.kind.is_composite() {
            bail!("Cannot and composite {}", self);
        }
        Ok(TypedBits {
            bits: bits_and(&self.bits, &rhs.bits),
            kind: self.kind,
        })
    }
}

impl std::ops::BitOr for TypedBits {
    type Output = Result<TypedBits>;

    fn bitor(self, rhs: TypedBits) -> Self::Output {
        if self.kind != rhs.kind {
            bail!(
                "Cannot or {} and {} because they have different types",
                self,
                rhs
            );
        }
        if self.kind.is_composite() {
            bail!("Cannot or composite {}", self);
        }
        Ok(TypedBits {
            bits: bits_or(&self.bits, &rhs.bits),
            kind: self.kind,
        })
    }
}

impl std::ops::Neg for TypedBits {
    type Output = Result<TypedBits>;

    fn neg(self) -> Self::Output {
        if !self.kind.is_signed() {
            bail!("Only signed values can be negated: {}", self);
        }
        Ok(TypedBits {
            bits: bit_neg(&self.bits),
            kind: self.kind,
        })
    }
}

impl std::ops::Shl<TypedBits> for TypedBits {
    type Output = Result<TypedBits>;

    fn shl(self, rhs: TypedBits) -> Self::Output {
        if self.kind.is_composite() {
            bail!("Cannot shift composite {}", self);
        }
        if !rhs.kind.is_unsigned() {
            bail!("Shift amount must be unsigned: {}", rhs);
        }
        let shift = rhs.as_i64()?;
        if shift >= self.bits.len() as i64 {
            bail!(
                "Shift amount {} is greater than the number of bits in {}",
                shift,
                self
            );
        }
        Ok(TypedBits {
            bits: bits_shl(&self.bits, shift),
            kind: self.kind,
        })
    }
}

impl std::ops::Shr<TypedBits> for TypedBits {
    type Output = Result<TypedBits>;

    fn shr(self, rhs: TypedBits) -> Self::Output {
        if self.kind.is_composite() {
            bail!("Cannot shift composite {}", self);
        }
        if !rhs.kind.is_unsigned() {
            bail!("Shift amount must be unsigned: {}", rhs);
        }
        let shift = rhs.as_i64()?;
        if shift >= self.bits.len() as i64 {
            bail!(
                "Shift amount {} is greater than the number of bits in {}",
                shift,
                self
            );
        }
        if self.kind.is_signed() {
            Ok(TypedBits {
                bits: bits_shr_signed(&self.bits, shift),
                kind: self.kind,
            })
        } else {
            Ok(TypedBits {
                bits: bits_shr(&self.bits, shift),
                kind: self.kind,
            })
        }
    }
}

impl std::cmp::PartialOrd for TypedBits {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.kind != other.kind {
            return None;
        }
        if self.kind.is_unsigned() {
            let mut a_as_u128 = 0;
            let mut b_as_u128 = 0;
            for ndx in 0..self.bits.len() {
                a_as_u128 |= (self.bits[ndx] as u128) << ndx;
                b_as_u128 |= (other.bits[ndx] as u128) << ndx;
            }
            a_as_u128.partial_cmp(&b_as_u128)
        } else {
            let mut a_as_i128 = 0;
            let mut b_as_i128 = 0;
            for ndx in 0..self.bits.len() {
                a_as_i128 |= (self.bits[ndx] as i128) << ndx;
                b_as_i128 |= (other.bits[ndx] as i128) << ndx;
            }
            let me_sign = self.bits.last().cloned().unwrap_or_default();
            let other_sign = other.bits.last().cloned().unwrap_or_default();
            for ndx in self.bits.len()..128 {
                a_as_i128 |= (me_sign as i128) << ndx;
                b_as_i128 |= (other_sign as i128) << ndx;
            }
            a_as_i128.partial_cmp(&b_as_i128)
        }
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

    #[test]
    fn test_typed_bits_add() {
        let a = 42_u8.typed_bits();
        let b = 196_u8.typed_bits();
        assert!(a < b);
        assert!(a <= b);
        assert!(b > a);
        assert!(b >= a);
        let c = (a + b).unwrap();
        assert_eq!(c, 238_u8.typed_bits());
    }
}
