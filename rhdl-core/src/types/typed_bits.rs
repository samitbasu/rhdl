use std::iter::repeat;

use crate::dyn_bit_manip::bits_shr_signed;
use crate::dyn_bit_manip::{
    bit_neg, bit_not, bits_and, bits_or, bits_shl, bits_shr, bits_xor, full_add, full_sub,
};
use crate::error::{rhdl_error, RHDLError};
use crate::types::bitx::bitx_string;
use crate::{
    types::path::{bit_range, Path},
    Kind,
};
use crate::{Color, VariantType};

use super::bitx::BitX;
use super::error::DynamicTypeError;
use super::kind::Array;
use super::kind::Enum;
use super::kind::Struct;
use super::kind::Tuple;

type Result<T> = std::result::Result<T, RHDLError>;

#[derive(Clone, PartialEq, Hash)]
pub struct TypedBits {
    pub bits: Vec<BitX>,
    pub kind: Kind,
}

impl From<i64> for TypedBits {
    fn from(mut val: i64) -> Self {
        let mut bits = Vec::new();
        for _ in 0..64 {
            bits.push((val & 1 != 0).into());
            val >>= 1;
        }
        TypedBits {
            bits,
            kind: Kind::make_signed(64),
        }
    }
}

impl From<u64> for TypedBits {
    fn from(mut val: u64) -> Self {
        let mut bits = Vec::new();
        for _ in 0..64 {
            bits.push((val & 1 != 0).into());
            val >>= 1;
        }
        TypedBits {
            bits,
            kind: Kind::make_bits(64),
        }
    }
}

impl TypedBits {
    pub const EMPTY: TypedBits = TypedBits {
        bits: Vec::new(),
        kind: Kind::Empty,
    };
    pub fn as_uninit(self) -> Self {
        TypedBits {
            bits: repeat(BitX::X).take(self.kind.bits()).collect(),
            kind: self.kind,
        }
    }
    pub fn is_initialized(&self) -> bool {
        self.bits.iter().all(|x| x.is_init())
    }
    pub fn path(&self, path: &Path) -> Result<TypedBits> {
        let (range, kind) = bit_range(self.kind.clone(), path)?;
        Ok(TypedBits {
            bits: self.bits[range].to_vec(),
            kind,
        })
    }
    pub fn splice(&self, path: &Path, value: TypedBits) -> Result<TypedBits> {
        let (range, kind) = bit_range(self.kind.clone(), path)?;
        if kind != value.kind {
            return Err(rhdl_error(DynamicTypeError::IllegalSplice {
                value,
                kind,
                path: path.clone(),
            }));
        }
        let mut new_bits = self.bits.clone();
        new_bits.splice(range, value.bits.iter().cloned());
        Ok(TypedBits {
            bits: new_bits,
            kind: self.kind.clone(),
        })
    }
    pub fn discriminant(&self) -> Result<TypedBits> {
        if self.kind.is_enum() {
            self.path(&Path::default().discriminant())
        } else {
            Ok(self.clone())
        }
    }
    pub fn is_unmatched_variant(&self) -> bool {
        if !self.kind.is_enum() {
            return false;
        }
        let Ok(discriminant) = self.discriminant() else {
            return false;
        };
        let Ok(discriminant) = discriminant.as_i64() else {
            return false;
        };
        if let Some(variant) = self.kind.lookup_variant(discriminant) {
            variant.ty == VariantType::Unmatched
        } else {
            false
        }
    }
    fn resize_unsigned(&self, bits: usize) -> TypedBits {
        if bits <= self.bits.len() {
            return TypedBits {
                bits: self.bits[..bits].to_vec(),
                kind: Kind::make_bits(bits),
            };
        }
        TypedBits {
            bits: self
                .bits
                .iter()
                .copied()
                .chain(repeat(BitX::Zero))
                .take(bits)
                .collect(),
            kind: Kind::make_bits(bits),
        }
    }
    fn resize_signed(&self, bits: usize) -> TypedBits {
        if bits <= self.bits.len() {
            return TypedBits {
                bits: self.bits[..bits].to_vec(),
                kind: Kind::make_signed(bits),
            };
        }
        let sign_bit = self.bits.last().cloned().unwrap_or_default();
        TypedBits {
            bits: self
                .bits
                .iter()
                .copied()
                .chain(repeat(sign_bit))
                .take(bits)
                .collect(),
            kind: Kind::make_signed(bits),
        }
    }
    pub fn resize(&self, bits: usize) -> Result<TypedBits> {
        if !self.is_initialized() {
            return Err(rhdl_error(DynamicTypeError::ResizeUninitializedValue {
                value: self.clone(),
                len: bits,
            }));
        }
        match &self.kind {
            Kind::Bits(_) => Ok(self.resize_unsigned(bits)),
            Kind::Signed(_) => Ok(self.resize_signed(bits)),
            _ => Err(rhdl_error(DynamicTypeError::ReinterpretCastFailed {
                value: self.clone(),
                len: bits,
            })),
        }
    }
    pub fn unsigned_cast(&self, bits: usize) -> Result<TypedBits> {
        if !self.is_initialized() {
            return Err(rhdl_error(DynamicTypeError::ResizeUninitializedValue {
                value: self.clone(),
                len: bits,
            }));
        }
        if bits > self.kind.bits() {
            return Ok(TypedBits {
                bits: self
                    .bits
                    .clone()
                    .into_iter()
                    .chain(repeat(BitX::Zero))
                    .take(bits)
                    .collect(),
                kind: Kind::make_bits(bits),
            });
        }
        let (base, rest) = self.bits.split_at(bits);
        if rest.iter().any(|b| b.maybe_one()) {
            return Err(rhdl_error(DynamicTypeError::UnsignedCastWithWidthFailed {
                value: self.clone(),
                bits,
            }));
        }
        Ok(TypedBits {
            bits: base.to_vec(),
            kind: Kind::make_bits(bits),
        })
    }
    pub fn signed_cast(&self, bits: usize) -> Result<TypedBits> {
        if !self.is_initialized() {
            return Err(rhdl_error(DynamicTypeError::ResizeUninitializedValue {
                value: self.clone(),
                len: bits,
            }));
        }
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
            return Err(rhdl_error(DynamicTypeError::SignedCastWithWidthFailed {
                value: self.clone(),
                bits,
            }));
        }
        Ok(TypedBits {
            bits: base.to_vec(),
            kind: Kind::make_signed(bits),
        })
    }
    pub fn as_i64(&self) -> Result<i64> {
        if !self.is_initialized() {
            return Err(rhdl_error(DynamicTypeError::UnableToInterpretAsI64 {
                kind: self.kind.clone(),
            }));
        }
        let tb64 = match &self.kind {
            Kind::Bits(_) => self.unsigned_cast(64)?,
            Kind::Signed(_) => self.signed_cast(64)?,
            Kind::Signal(base, _) if base.is_unsigned() => self.unsigned_cast(64)?,
            Kind::Signal(base, _) if base.is_signed() => self.signed_cast(64)?,
            _ => {
                return Err(rhdl_error(DynamicTypeError::UnableToInterpretAsI64 {
                    kind: self.kind.clone(),
                }));
            }
        };
        let mut ret: u64 = 0;
        for ndx in 0..64 {
            ret |= (tb64.bits[ndx] as u64) << ndx;
        }
        Ok(ret as i64)
    }
    pub fn any(&self) -> TypedBits {
        let any = self.bits.iter().fold(BitX::Zero, |a, b| a | *b);
        TypedBits {
            bits: vec![any],
            kind: Kind::make_bool(),
        }
    }
    pub fn all(&self) -> TypedBits {
        let all = self.bits.iter().fold(BitX::One, |a, b| a & *b);
        TypedBits {
            bits: vec![all],
            kind: Kind::make_bool(),
        }
    }
    pub fn as_signed(&self) -> Result<TypedBits> {
        if !self.is_initialized() {
            return Err(rhdl_error(DynamicTypeError::SignedCastFailed {
                value: self.clone(),
            }));
        }
        if let Kind::Bits(ndx) = self.kind {
            Ok(TypedBits {
                bits: self.bits.clone(),
                kind: Kind::Signed(ndx),
            })
        } else {
            Err(rhdl_error(DynamicTypeError::SignedCastFailed {
                value: self.clone(),
            }))
        }
    }
    pub fn as_unsigned(&self) -> Result<TypedBits> {
        if !self.is_initialized() {
            return Err(rhdl_error(DynamicTypeError::UnsignedCastFailed {
                value: self.clone(),
            }));
        }
        if let Kind::Signed(ndx) = self.kind {
            Ok(TypedBits {
                bits: self.bits.clone(),
                kind: Kind::Bits(ndx),
            })
        } else {
            Err(rhdl_error(DynamicTypeError::UnsignedCastFailed {
                value: self.clone(),
            }))
        }
    }
    pub fn sign_bit(&self) -> Result<TypedBits> {
        if !self.is_initialized() {
            return Err(rhdl_error(DynamicTypeError::CannotGetSignBit {
                value: self.clone(),
            }));
        }
        if self.kind.is_signed() {
            Ok(TypedBits {
                bits: vec![self.bits.last().cloned().unwrap_or_default()],
                kind: Kind::make_bits(1),
            })
        } else {
            Err(rhdl_error(DynamicTypeError::CannotGetSignBit {
                value: self.clone(),
            }))
        }
    }
    pub fn xor(&self) -> TypedBits {
        TypedBits {
            bits: vec![self.bits.iter().fold(BitX::Zero, |a, b| a ^ *b)],
            kind: Kind::make_bool(),
        }
    }
    pub fn as_bool(&self) -> Result<bool> {
        if self.kind.is_bool() {
            self.bits[0].try_into()
        } else {
            Err(rhdl_error(DynamicTypeError::CannotCastToBool {
                value: self.clone(),
            }))
        }
    }
    pub fn repeat(&self, count: usize) -> TypedBits {
        let my_len = self.bits.len();
        TypedBits {
            bits: self
                .bits
                .iter()
                .cloned()
                .cycle()
                .take(count * my_len)
                .collect(),
            kind: Kind::make_array(self.kind.clone(), count),
        }
    }
    pub fn slice(&self, offset: usize, count: usize) -> Result<TypedBits> {
        if self.kind.is_composite() {
            return Err(rhdl_error(DynamicTypeError::CannotSliceComposite {
                value: self.clone(),
            }));
        }
        if offset + count > self.bits.len() {
            return Err(rhdl_error(DynamicTypeError::CannotSliceBits {
                start: offset,
                end: offset + count,
                value: self.clone(),
            }));
        }
        Ok(TypedBits {
            bits: self.bits[offset..offset + count].to_vec(),
            kind: Kind::make_bits(count),
        })
    }
    pub fn with_clock(self, color: Color) -> TypedBits {
        TypedBits {
            bits: self.bits,
            kind: Kind::make_signal(self.kind, color),
        }
    }

    pub fn val(&self) -> TypedBits {
        TypedBits {
            bits: self.bits.clone(),
            kind: self.kind.val(),
        }
    }
    pub fn as_verilog_literal(self) -> String {
        let signed = if matches!(self.kind, Kind::Signed(_)) {
            "s"
        } else {
            ""
        };
        let width = self.bits.len();
        let bs = bitx_string(&self.bits);
        format!("{width}'{signed}b{bs}")
    }
}

fn binop_kind(lhs: &Kind, rhs: &Kind) -> Result<Kind> {
    if lhs.is_composite() || rhs.is_composite() {
        return Err(rhdl_error(
            DynamicTypeError::CannotApplyBinaryOperationToComposite {
                value: TypedBits::EMPTY,
            },
        ));
    }
    if lhs == rhs {
        return Ok(lhs.clone());
    }
    let signal_kind = lhs.signal_data();
    let Some(clock) = lhs.signal_clock().or(rhs.signal_clock()) else {
        return Err(rhdl_error(
            DynamicTypeError::BinaryOperationRequiresCompatibleType {
                lhs: lhs.clone(),
                rhs: rhs.clone(),
            },
        ));
    };
    Ok(Kind::make_signal(signal_kind, clock))
}

impl std::ops::Add<TypedBits> for TypedBits {
    type Output = Result<TypedBits>;

    fn add(self, rhs: TypedBits) -> Self::Output {
        Ok(TypedBits {
            bits: full_add(&self.bits, &rhs.bits),
            kind: binop_kind(&self.kind, &rhs.kind)?,
        })
    }
}

impl std::ops::Sub<TypedBits> for TypedBits {
    type Output = Result<TypedBits>;

    fn sub(self, rhs: TypedBits) -> Self::Output {
        Ok(TypedBits {
            bits: full_sub(&self.bits, &rhs.bits),
            kind: binop_kind(&self.kind, &rhs.kind)?,
        })
    }
}

impl std::ops::Not for TypedBits {
    type Output = Result<TypedBits>;

    fn not(self) -> Self::Output {
        if self.kind.is_composite() {
            return Err(rhdl_error(DynamicTypeError::CannotNegateComposite {
                value: self.clone(),
            }));
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
        Ok(TypedBits {
            bits: bits_xor(&self.bits, &rhs.bits),
            kind: binop_kind(&self.kind, &rhs.kind)?,
        })
    }
}

impl std::ops::BitAnd for TypedBits {
    type Output = Result<TypedBits>;

    fn bitand(self, rhs: TypedBits) -> Self::Output {
        Ok(TypedBits {
            bits: bits_and(&self.bits, &rhs.bits),
            kind: binop_kind(&self.kind, &rhs.kind)?,
        })
    }
}

impl std::ops::BitOr for TypedBits {
    type Output = Result<TypedBits>;

    fn bitor(self, rhs: TypedBits) -> Self::Output {
        Ok(TypedBits {
            bits: bits_or(&self.bits, &rhs.bits),
            kind: binop_kind(&self.kind, &rhs.kind)?,
        })
    }
}

impl std::ops::Neg for TypedBits {
    type Output = Result<TypedBits>;

    fn neg(self) -> Self::Output {
        if !self.kind.is_signed() {
            return Err(rhdl_error(DynamicTypeError::CannotNegateUnsigned {
                value: self.clone(),
            }));
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
            return Err(rhdl_error(
                DynamicTypeError::CannotApplyShiftOperationToComposite {
                    value: self.clone(),
                },
            ));
        }
        if !rhs.kind.is_unsigned() {
            return Err(rhdl_error(DynamicTypeError::ShiftAmountMustBeUnsigned {
                value: rhs.clone(),
            }));
        }
        let shift = rhs.as_i64()?;
        if shift >= self.bits.len() as i64 {
            return Err(rhdl_error(DynamicTypeError::ShiftAmountMustBeLessThan {
                value: rhs.clone(),
                max: self.bits.len(),
            }));
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
            return Err(rhdl_error(
                DynamicTypeError::CannotApplyShiftOperationToComposite {
                    value: self.clone(),
                },
            ));
        }
        if !rhs.kind.is_unsigned() {
            return Err(rhdl_error(DynamicTypeError::ShiftAmountMustBeUnsigned {
                value: rhs.clone(),
            }));
        }
        let shift = rhs.as_i64()?;
        if shift >= self.bits.len() as i64 {
            return Err(rhdl_error(DynamicTypeError::ShiftAmountMustBeLessThan {
                value: rhs.clone(),
                max: self.bits.len(),
            }));
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
        let is_unsigned = if let Some(kind) = self.kind.signal_kind() {
            kind.is_unsigned()
        } else {
            self.kind.is_unsigned()
        };
        if is_unsigned {
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

impl std::fmt::Debug for TypedBits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_kind_with_bits(&self.kind, &self.bits, f)
    }
}

fn write_kind_with_bits(
    kind: &Kind,
    bits: &[BitX],
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    match kind {
        Kind::Array(array) => write_array(array, bits, f),
        Kind::Tuple(tuple) => write_tuple(tuple, bits, f),
        Kind::Struct(structure) => write_struct(structure, bits, f),
        Kind::Enum(enumerate) => write_enumerate(enumerate, bits, f),
        Kind::Bits(_) => write_bits(bits, f),
        Kind::Signed(_) => write_signed(bits, f),
        Kind::Empty => write!(f, "()"),
        Kind::Signal(base, color) => {
            write_kind_with_bits(base, bits, f)?;
            write!(f, "@{:?}", color)
        }
    }
}

fn interpret_bits_as_i64(bits: &[BitX], signed: bool) -> Result<i64> {
    let bits = bits
        .iter()
        .map(|&x| x.try_into())
        .collect::<Result<Vec<bool>>>()?;
    // If the value is signed, then we sign extend it to 128 bits
    let value = if signed {
        let sign = bits.last().copied().unwrap_or_default();
        repeat(&sign)
            .take(128 - bits.len())
            .chain(bits.iter().rev())
            .fold(0_i128, |acc, b| (acc << 1) | (*b as i128))
    } else {
        bits.iter()
            .rev()
            .fold(0_u128, |acc, b| (acc << 1) | (*b as u128)) as i128
    };
    Ok(value as i64)
}

fn write_enumerate(
    enumerate: &Enum,
    bits: &[BitX],
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let root_kind = Kind::Enum(enumerate.clone());
    let (range, kind) = bit_range(root_kind.clone(), &Path::default().discriminant()).unwrap();
    let discriminant_value = match interpret_bits_as_i64(&bits[range], kind.is_signed()) {
        Ok(val) => val,
        Err(_) => return write!(f, "{}::??", enumerate.name),
    };
    // Get the variant for this discriminant
    let variant = enumerate
        .variants
        .iter()
        .find(|v| v.discriminant == discriminant_value)
        .unwrap();
    write!(f, "{}::{}", enumerate.name, variant.name)?;
    let (payload_range, payload_kind) = bit_range(
        root_kind,
        &Path::default().payload_by_value(discriminant_value),
    )
    .unwrap();
    if payload_range.is_empty() {
        return Ok(());
    }
    let payload = &bits[payload_range];
    write_kind_with_bits(&payload_kind, payload, f)
}

fn write_struct(
    structure: &Struct,
    bits: &[BitX],
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    write!(f, "{} {{", structure.name)?;
    let root_kind = Kind::Struct(structure.clone());
    for (ndx, field) in structure.fields.iter().enumerate() {
        let (bit_range, sub_kind) =
            bit_range(root_kind.clone(), &Path::default().field(&field.name)).unwrap();
        let slice = &bits[bit_range];
        write!(f, "{}: ", field.name)?;
        write_kind_with_bits(&sub_kind, slice, f)?;
        if ndx < structure.fields.len() - 1 {
            write!(f, ", ")?;
        }
    }
    write!(f, "}}")
}

fn write_array(array: &Array, bits: &[BitX], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[")?;
    let root_kind = Kind::Array(array.clone());
    for ndx in 0..(array.size) {
        let (bit_range, sub_kind) =
            bit_range(root_kind.clone(), &Path::default().index(ndx)).unwrap();
        let slice = &bits[bit_range];
        write_kind_with_bits(&sub_kind, slice, f)?;
        if ndx < array.size - 1 {
            write!(f, ", ")?;
        }
    }
    write!(f, "]")
}

fn write_bits(bits: &[BitX], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if bits.len() == 1 {
        return write!(
            f,
            "{}",
            match bits[0] {
                BitX::Zero => "false",
                BitX::One => "true",
                BitX::X => "x",
            }
        );
    }
    write!(f, "{}_b{}", bitx_string(bits), bits.len())
}

fn write_signed(bits: &[BitX], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if bits.len() == 1 {
        return write!(
            f,
            "{}",
            if bits[0].is_one() {
                "-1".to_string()
            } else {
                bits[0].to_string()
            }
        );
    }
    let bools = bits
        .iter()
        .map(|&x| x.try_into())
        .collect::<Result<Vec<bool>>>();
    match bools {
        Ok(bits) => {
            // We know that the bits array will fit into a i128.
            let bit_len = bits.len();
            let sign_bit = bits.last().cloned().unwrap_or_default();
            let val = repeat(&sign_bit)
                .take(128 - bit_len)
                .chain(bits.iter().rev())
                .fold(0_i128, |acc, b| (acc << 1_i128) | (*b as i128));
            write!(f, "{}_s{}", val, bits.len())
        }
        Err(_) => {
            write!(f, "{}_s{}", bitx_string(bits), bits.len())
        }
    }
}

fn write_tuple(tuple: &Tuple, bits: &[BitX], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "(")?;
    let root_kind = Kind::Tuple(tuple.clone());
    for ndx in 0..(tuple.elements.len()) {
        let (bit_range, sub_kind) =
            bit_range(root_kind.clone(), &Path::default().tuple_index(ndx)).unwrap();
        let slice = &bits[bit_range];
        write_kind_with_bits(&sub_kind, slice, f)?;
        if ndx < tuple.elements.len() - 1 {
            write!(f, ", ")?;
        }
    }
    write!(f, ")")
}

#[cfg(test)]
mod tests {
    use rand::thread_rng;

    use crate::{Digital, DiscriminantAlignment, DiscriminantType, Kind, Notable, TypedBits};

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

    #[test]
    #[allow(dead_code)]
    #[allow(clippy::just_underscores_and_digits)]
    fn test_display_typed_bits() {
        #[derive(Debug, Clone, PartialEq, Copy)]
        enum Baz {
            A(Bar),
            B { foo: Foo },
            C(u8),
        }

        impl Default for Baz {
            fn default() -> Self {
                Self::A(Default::default())
            }
        }

        impl Notable for Baz {
            fn note(&self, _: impl crate::NoteKey, _: impl crate::NoteWriter) {
                todo!()
            }
        }

        impl Digital for Baz {
            fn static_kind() -> Kind {
                Kind::make_enum(
                    concat!(module_path!(), "::", stringify!(Baz)),
                    vec![
                        Kind::make_variant(
                            stringify!(A),
                            Kind::make_tuple(vec![<Bar as Digital>::static_kind()]),
                            0i64,
                            crate::VariantType::Normal,
                        ),
                        Kind::make_variant(
                            stringify!(B),
                            Kind::make_struct(
                                stringify!(_Baz__B),
                                vec![Kind::make_field(
                                    stringify!(foo),
                                    <Foo as Digital>::static_kind(),
                                )],
                            ),
                            1i64,
                            crate::VariantType::Normal,
                        ),
                        Kind::make_variant(
                            stringify!(C),
                            Kind::make_tuple(vec![<u8 as Digital>::static_kind()]),
                            2i64,
                            crate::VariantType::Normal,
                        ),
                    ],
                    Kind::make_discriminant_layout(
                        2usize,
                        DiscriminantAlignment::Msb,
                        DiscriminantType::Unsigned,
                    ),
                )
            }
            fn bin(self) -> Vec<bool> {
                self.kind().pad(match self {
                    Self::A(_0) => {
                        let mut v = rhdl_bits::bits::<2usize>(0i64 as u128).to_bools();
                        v.extend(_0.bin());
                        v
                    }
                    Self::B { foo } => {
                        let mut v = rhdl_bits::bits::<2usize>(1i64 as u128).to_bools();
                        v.extend(foo.bin());
                        v
                    }
                    Self::C(_0) => {
                        let mut v = rhdl_bits::bits::<2usize>(2i64 as u128).to_bools();
                        v.extend(_0.bin());
                        v
                    }
                })
            }
            fn discriminant(self) -> TypedBits {
                match self {
                    Self::A(_0) => rhdl_bits::bits::<2usize>(0i64 as u128).typed_bits(),
                    Self::B { foo: _ } => rhdl_bits::bits::<2usize>(1i64 as u128).typed_bits(),
                    Self::C(_0) => rhdl_bits::bits::<2usize>(2i64 as u128).typed_bits(),
                }
            }
            fn variant_kind(self) -> Kind {
                match self {
                    Self::A(_0) => Kind::make_tuple(vec![<Bar as Digital>::static_kind()]),
                    Self::B { foo: _ } => Kind::make_struct(
                        stringify!(_Baz__B),
                        vec![Kind::make_field(
                            stringify!(foo),
                            <Foo as Digital>::static_kind(),
                        )],
                    ),
                    Self::C(_0) => Kind::make_tuple(vec![<u8 as Digital>::static_kind()]),
                }
            }
            fn uninit() -> Self {
                use rand::Rng;
                match rand::thread_rng().gen_range(0..3) {
                    0 => Self::A(Default::default()),
                    1 => Self::B {
                        foo: Default::default(),
                    },
                    2 => Self::C(thread_rng().gen()),
                    _ => unreachable!(),
                }
            }
        }

        #[derive(Debug, Clone, PartialEq, Copy, Default)]
        struct Bar(u8, u8, bool);

        impl Notable for Bar {
            fn note(&self, _key: impl crate::NoteKey, _writer: impl crate::NoteWriter) {
                todo!()
            }
        }

        impl Digital for Bar {
            fn static_kind() -> Kind {
                Kind::make_struct(
                    "Bar",
                    vec![
                        Kind::make_field("0", Kind::Bits(8)),
                        Kind::make_field("1", Kind::Bits(8)),
                        Kind::make_field("2", Kind::Bits(1)),
                    ],
                )
            }
            fn bin(self) -> Vec<bool> {
                [self.0.bin(), self.1.bin(), self.2.bin()].concat()
            }
            fn uninit() -> Self {
                use rand::Rng;
                Self(
                    rand::thread_rng().gen(),
                    rand::thread_rng().gen(),
                    rand::thread_rng().gen(),
                )
            }
        }

        #[derive(Debug, Clone, PartialEq, Copy, Default)]
        struct Foo {
            a: u8,
            b: u8,
            c: bool,
        }
        impl Notable for Foo {
            fn note(&self, _key: impl crate::NoteKey, _writer: impl crate::NoteWriter) {
                todo!()
            }
        }

        impl Digital for Foo {
            fn static_kind() -> Kind {
                Kind::make_struct(
                    "Foo",
                    vec![
                        Kind::make_field("a", Kind::Bits(8)),
                        Kind::make_field("b", Kind::Bits(8)),
                        Kind::make_field("c", Kind::Bits(1)),
                    ],
                )
            }
            fn bin(self) -> Vec<bool> {
                [self.a.bin(), self.b.bin(), self.c.bin()].concat()
            }
            fn uninit() -> Self {
                use rand::Rng;
                Self {
                    a: rand::thread_rng().gen(),
                    b: rand::thread_rng().gen(),
                    c: rand::thread_rng().gen(),
                }
            }
        }

        let a = 0x47_u8.typed_bits();
        assert_eq!(format!("{:?}", a), "47_b8");
        let c = (0x12_u8, 0x80_u8, false).typed_bits();
        assert_eq!(format!("{:?}", c), "(12_b8, 80_b8, false)");
        let b = (-0x53_i32).typed_bits();
        assert_eq!(format!("{:?}", b), "-83_s32");
        let d = [1_u8, 3_u8, 4_u8].typed_bits();
        assert_eq!(format!("{:?}", d), "[1_b8, 3_b8, 4_b8]");
        let e = Foo {
            a: 0x47,
            b: 0x80,
            c: true,
        }
        .typed_bits();
        assert_eq!(format!("{:?}", e), "Foo {a: 47_b8, b: 80_b8, c: true}");
        let e = Bar(0x47, 0x80, true).typed_bits();
        assert_eq!(format!("{:?}", e), "Bar {0: 47_b8, 1: 80_b8, 2: true}");
        let d = [Bar(0x47, 0x80, true), Bar(0x42, 0x13, false)].typed_bits();
        assert_eq!(
            format!("{:?}", d),
            "[Bar {0: 47_b8, 1: 80_b8, 2: true}, Bar {0: 42_b8, 1: 13_b8, 2: false}]"
        );
        let h = Baz::A(Bar(0x47, 0x80, true)).typed_bits();
        assert_eq!(
            format!("{:?}", h),
            "rhdl_core::types::typed_bits::tests::Baz::A(Bar {0: 47_b8, 1: 80_b8, 2: true})"
        );
    }

    #[test]
    fn test_add_signals() {
        let a = 42_u8.typed_bits().with_clock(crate::Color::Red);
        let b = 196_u8.typed_bits().with_clock(crate::Color::Red);
        let c = (a + b).unwrap();
        assert_eq!(c.kind, Kind::make_signal(Kind::Bits(8), crate::Color::Red));
        assert_eq!(c.bits, 238_u8.typed_bits().bits);
    }
}
