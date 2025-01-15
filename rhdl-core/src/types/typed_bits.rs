use std::iter::{once, repeat};

use internment::Intern;

use crate::ast::ast_impl::WrapOp;
use crate::bitx::dyn_bit_manip::bits_shr_signed;
use crate::bitx::dyn_bit_manip::{
    bit_neg, bit_not, bits_and, bits_or, bits_shl, bits_shr, bits_xor, full_add, full_sub,
};
use crate::bitx::{bitx_string, BitX};
use crate::error::{rhdl_error, RHDLError};
use crate::Color;
use crate::{
    types::path::{bit_range, sub_kind, Path},
    Kind,
};

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

impl From<BitX> for TypedBits {
    fn from(value: BitX) -> Self {
        TypedBits {
            bits: vec![value],
            kind: Kind::make_bits(1),
        }
    }
}

impl TypedBits {
    pub const EMPTY: TypedBits = TypedBits {
        bits: Vec::new(),
        kind: Kind::Empty,
    };
    pub fn dont_care_from_kind(kind: Kind) -> Self {
        Self {
            bits: vec![BitX::X; kind.bits()],
            kind,
        }
    }
    pub fn dont_care(self) -> Self {
        Self {
            bits: vec![BitX::X; self.bits.len()],
            ..self
        }
    }
    pub fn path(&self, path: &Path) -> Result<TypedBits> {
        let (range, kind) = bit_range(self.kind, path)?;
        Ok(TypedBits {
            bits: self.bits[range].to_vec(),
            kind,
        })
    }
    pub fn splice(&self, path: &Path, value: TypedBits) -> Result<TypedBits> {
        let (range, kind) = bit_range(self.kind, path)?;
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
            kind: self.kind,
        })
    }
    pub fn discriminant(&self) -> Result<TypedBits> {
        if self.kind.is_enum() {
            self.path(&Path::default().discriminant())
        } else {
            Ok(self.clone())
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
        let sign_bit = self.bits.last().cloned().unwrap_or(BitX::Zero);
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
        match &self.kind {
            Kind::Bits(_) => Ok(self.resize_unsigned(bits)),
            Kind::Signed(_) => Ok(self.resize_signed(bits)),
            _ => Err(rhdl_error(DynamicTypeError::ReinterpretCastFailed {
                value: self.clone(),
                len: bits,
            })),
        }
    }
    pub fn xext(&self, len: usize) -> Result<TypedBits> {
        self.resize(self.kind.bits() + len)
    }
    pub fn xshl(&self, len: usize) -> Result<TypedBits> {
        let kind = match &self.kind {
            Kind::Bits(n) => Kind::make_bits(n + len),
            Kind::Signed(n) => Kind::make_signed(n + len),
            _ => {
                return Err(rhdl_error(DynamicTypeError::XshlFailed {
                    value: self.clone(),
                    len,
                }))
            }
        };
        let bits = repeat(BitX::Zero)
            .take(len)
            .chain(self.bits.iter().copied())
            .collect();
        Ok(TypedBits { bits, kind })
    }
    pub fn xshr(&self, len: usize) -> Result<TypedBits> {
        let kind = match &self.kind {
            Kind::Bits(n) => Kind::make_bits(n - len),
            Kind::Signed(n) => Kind::make_signed(n - len),
            _ => {
                return Err(rhdl_error(DynamicTypeError::XshrFailed {
                    value: self.clone(),
                    len,
                }))
            }
        };
        let bits = self.bits.iter().copied().skip(len).collect();
        Ok(TypedBits { bits, kind })
    }
    pub fn unsigned_cast(&self, bits: usize) -> Result<TypedBits> {
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
        if rest.iter().fold(BitX::Zero, |acc, b| acc | *b) != BitX::Zero {
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
        if bits > self.kind.bits() {
            let sign_bit = self.bits.last().cloned().unwrap_or(BitX::Zero);
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
        let new_sign_bit = base.last().cloned().unwrap_or(BitX::Zero);
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
        let tb64 = match &self.kind {
            Kind::Bits(_) => self.unsigned_cast(64)?,
            Kind::Signed(_) => self.signed_cast(64)?,
            Kind::Signal(base, _) if base.is_unsigned() => self.unsigned_cast(64)?,
            Kind::Signal(base, _) if base.is_signed() => self.signed_cast(64)?,
            _ => {
                return Err(rhdl_error(DynamicTypeError::UnableToInterpretAsI64 {
                    kind: self.kind,
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
        self.bits.iter().fold(BitX::Zero, |a, b| a | *b).into()
    }
    pub fn all(&self) -> TypedBits {
        self.bits.iter().fold(BitX::One, |a, b| a & *b).into()
    }
    pub fn as_signed(&self) -> Result<TypedBits> {
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
        if self.kind.is_signed() {
            Ok(self.bits.last().cloned().unwrap_or(BitX::Zero).into())
        } else {
            Err(rhdl_error(DynamicTypeError::CannotGetSignBit {
                value: self.clone(),
            }))
        }
    }
    pub fn xor(&self) -> TypedBits {
        self.bits.iter().fold(BitX::Zero, |a, b| a ^ *b).into()
    }
    pub fn as_bool(&self) -> Result<bool> {
        if self.kind.is_bool() {
            match self.bits[0] {
                BitX::Zero => Ok(false),
                BitX::One => Ok(true),
                BitX::X => Err(rhdl_error(DynamicTypeError::CannotCoerceUninitToBool {
                    value: self.clone(),
                })),
            }
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
            kind: Kind::make_array(self.kind, count),
        }
    }
    pub fn get_bit(&self, index: usize) -> Result<TypedBits> {
        if index >= self.bits.len() {
            return Err(rhdl_error(DynamicTypeError::CannotGetBit {
                ndx: index,
                value: self.clone(),
            }));
        }
        Ok(TypedBits {
            bits: vec![self.bits[index]],
            kind: Kind::make_bits(1),
        })
    }
    pub fn set_bit(&self, index: usize, val: bool) -> Result<TypedBits> {
        if index >= self.bits.len() {
            return Err(rhdl_error(DynamicTypeError::CannotSetBit {
                ndx: index,
                value: self.clone(),
                bit: val,
            }));
        }
        if self.kind.is_composite() {
            return Err(rhdl_error(DynamicTypeError::CannotSetBitOnComposite {
                value: self.clone(),
            }));
        }
        let mut new_bits = self.bits.clone();
        new_bits[index] = val.into();
        Ok(TypedBits {
            bits: new_bits,
            kind: self.kind,
        })
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
    pub fn wrap_some(self, option_kind: &Kind) -> Result<TypedBits> {
        if !option_kind.is_option() {
            return Err(rhdl_error(DynamicTypeError::CannotWrapOption {
                value: self.clone(),
                kind: *option_kind,
            }));
        }
        let some_kind = sub_kind(
            *option_kind,
            &Path::default().payload("Some").tuple_index(0),
        )?;
        if some_kind != self.kind {
            return Err(rhdl_error(DynamicTypeError::CannotWrapOption {
                value: self.clone(),
                kind: *option_kind,
            }));
        }
        let pad = option_kind.bits() - self.kind.bits() - 1;
        Ok(TypedBits {
            bits: self
                .bits
                .into_iter()
                .chain(repeat(BitX::Zero).take(pad).chain(once(BitX::One)))
                .collect(),
            kind: *option_kind,
        })
    }
    pub fn wrap_none(self, option_kind: &Kind) -> Result<TypedBits> {
        if !option_kind.is_option() {
            return Err(rhdl_error(DynamicTypeError::CannotWrapOption {
                value: self.clone(),
                kind: *option_kind,
            }));
        }
        let none_kind = sub_kind(*option_kind, &Path::default().payload("None"))?;
        if none_kind != self.kind {
            return Err(rhdl_error(DynamicTypeError::CannotWrapOption {
                value: self.clone(),
                kind: *option_kind,
            }));
        }
        let pad = option_kind.bits() - self.kind.bits() - 1;
        Ok(TypedBits {
            bits: self
                .bits
                .into_iter()
                .chain(repeat(BitX::Zero).take(pad).chain(once(BitX::Zero)))
                .collect(),
            kind: *option_kind,
        })
    }
    pub fn wrap_err(self, result_kind: &Kind) -> Result<TypedBits> {
        if !result_kind.is_result() {
            return Err(rhdl_error(DynamicTypeError::CannotWrapResult {
                value: self.clone(),
                kind: *result_kind,
            }));
        }
        let err_kind = sub_kind(*result_kind, &Path::default().payload("Err").tuple_index(0))?;
        if err_kind != self.kind {
            return Err(rhdl_error(DynamicTypeError::CannotWrapResult {
                value: self.clone(),
                kind: *result_kind,
            }));
        }
        let pad = result_kind.bits() - self.kind.bits() - 1;
        Ok(TypedBits {
            bits: self
                .bits
                .into_iter()
                .chain(repeat(BitX::Zero).take(pad).chain(once(BitX::Zero)))
                .collect(),
            kind: *result_kind,
        })
    }
    pub fn wrap_ok(self, result_kind: &Kind) -> Result<TypedBits> {
        if !result_kind.is_result() {
            return Err(rhdl_error(DynamicTypeError::CannotWrapResult {
                value: self.clone(),
                kind: *result_kind,
            }));
        }
        let ok_kind = sub_kind(*result_kind, &Path::default().payload("Ok").tuple_index(0))?;
        if ok_kind != self.kind {
            return Err(rhdl_error(DynamicTypeError::CannotWrapResult {
                value: self.clone(),
                kind: *result_kind,
            }));
        }
        let pad = result_kind.bits() - self.kind.bits() - 1;
        Ok(TypedBits {
            bits: self
                .bits
                .into_iter()
                .chain(repeat(BitX::Zero).take(pad).chain(once(BitX::One)))
                .collect(),
            kind: *result_kind,
        })
    }
    pub fn wrap(self, op: WrapOp, kind: &Kind) -> Result<TypedBits> {
        match op {
            WrapOp::Some => self.wrap_some(kind),
            WrapOp::None => self.wrap_none(kind),
            WrapOp::Err => self.wrap_err(kind),
            WrapOp::Ok => self.wrap_ok(kind),
        }
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
        return Ok(*lhs);
    }
    let signal_kind = lhs.signal_data();
    let Some(clock) = lhs.signal_clock().or(rhs.signal_clock()) else {
        return Err(rhdl_error(
            DynamicTypeError::BinaryOperationRequiresCompatibleType {
                lhs: *lhs,
                rhs: *rhs,
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
            let me_sign = self.bits.last().cloned().unwrap_or(BitX::Zero);
            let other_sign = other.bits.last().cloned().unwrap_or(BitX::Zero);
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

fn interpret_bits_as_i64(bits: &[BitX], signed: bool) -> i64 {
    // If the value is signed, then we sign extend it to 128 bits
    let value = if signed {
        let sign = bits.last().copied().unwrap_or(BitX::Zero);
        repeat(&sign)
            .take(128 - bits.len())
            .chain(bits.iter().rev())
            .fold(0_i128, |acc, b| (acc << 1) | (*b as i128))
    } else {
        bits.iter()
            .rev()
            .fold(0_u128, |acc, b| (acc << 1) | (*b as u128)) as i128
    };
    value as i64
}

fn write_enumerate(
    enumerate: &Enum,
    bits: &[BitX],
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let root_kind = Kind::Enum(Intern::new(enumerate.clone()));
    let (range, kind) = bit_range(root_kind, &Path::default().discriminant()).unwrap();
    let discriminant_value = interpret_bits_as_i64(&bits[range], kind.is_signed());
    // Get the variant for this discriminant
    if let Some(variant) = enumerate
        .variants
        .iter()
        .find(|v| v.discriminant == discriminant_value)
    {
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
    } else {
        write!(f, "{}", bitx_string(bits))
    }
}

fn write_struct(
    structure: &Struct,
    bits: &[BitX],
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    write!(f, "{} {{", structure.name)?;
    let root_kind = Kind::Struct(Intern::new(structure.clone()));
    for (ndx, field) in structure.fields.iter().enumerate() {
        let (bit_range, sub_kind) =
            bit_range(root_kind, &Path::default().field(&field.name)).unwrap();
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
    let root_kind = Kind::Array(Intern::new(array.clone()));
    for ndx in 0..(array.size) {
        let (bit_range, sub_kind) = bit_range(root_kind, &Path::default().index(ndx)).unwrap();
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
                BitX::X => "X",
            }
        );
    }
    // We know that the bits array will fit into a u128.
    let val = bits
        .iter()
        .rev()
        .fold(0_u128, |acc, b| (acc << 1) | (*b as u128));
    write!(f, "{:x}_b{}", val, bits.len())
}

fn write_signed(bits: &[BitX], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if bits.len() == 1 {
        return write!(
            f,
            "{}",
            match bits[0] {
                BitX::One => "-1",
                BitX::Zero => "0",
                BitX::X => "X",
            }
        );
    }
    // We know that the bits array will fit into a i128.
    let bit_len = bits.len();
    let sign_bit = bits.last().cloned().unwrap_or(BitX::Zero);
    let val = repeat(&sign_bit)
        .take(128 - bit_len)
        .chain(bits.iter().rev())
        .fold(0_i128, |acc, b| (acc << 1_i128) | (*b as i128));
    write!(f, "{}_s{}", val, bits.len())
}

fn write_tuple(tuple: &Tuple, bits: &[BitX], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "(")?;
    let root_kind = Kind::Tuple(Intern::new(tuple.clone()));
    for ndx in 0..(tuple.elements.len()) {
        let (bit_range, sub_kind) =
            bit_range(root_kind, &Path::default().tuple_index(ndx)).unwrap();
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
    use rhdl_bits::{alias::*, bits, consts::U2};

    use crate::{
        bitx::{bitx_vec, BitX},
        Digital, DiscriminantAlignment, DiscriminantType, Kind, TypedBits,
    };

    #[test]
    fn test_typed_bits_add() {
        let a = b8(42).typed_bits();
        let b = b8(196).typed_bits();
        assert!(a < b);
        assert!(a <= b);
        assert!(b > a);
        assert!(b >= a);
        let c = (a + b).unwrap();
        assert_eq!(c, b8(238).typed_bits());
    }

    #[test]
    #[allow(dead_code)]
    #[allow(clippy::just_underscores_and_digits)]
    fn test_display_typed_bits() {
        #[derive(Debug, Clone, PartialEq, Copy)]
        enum Baz {
            A(Bar),
            B { foo: Foo },
            C(b8),
        }

        impl Default for Baz {
            fn default() -> Self {
                Self::A(Default::default())
            }
        }

        impl Digital for Baz {
            const BITS: usize = 19;
            fn static_kind() -> Kind {
                Kind::make_enum(
                    concat!(module_path!(), "::", stringify!(Baz)),
                    vec![
                        Kind::make_variant(
                            stringify!(A),
                            Kind::make_tuple(vec![<Bar as Digital>::static_kind()]),
                            0i64,
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
                        ),
                        Kind::make_variant(
                            stringify!(C),
                            Kind::make_tuple(vec![<b8 as Digital>::static_kind()]),
                            2i64,
                        ),
                    ],
                    Kind::make_discriminant_layout(
                        2usize,
                        DiscriminantAlignment::Msb,
                        DiscriminantType::Unsigned,
                    ),
                )
            }
            fn static_trace_type() -> rhdl_trace_type::TraceType {
                crate::rtt::test::kind_to_trace(&Self::static_kind())
            }
            fn bin(self) -> Vec<BitX> {
                self.kind().pad(match self {
                    Self::A(_0) => {
                        let mut v = bitx_vec(&rhdl_bits::bits::<U2>(0i64 as u128).to_bools());
                        v.extend(_0.bin());
                        v
                    }
                    Self::B { foo } => {
                        let mut v = bitx_vec(&rhdl_bits::bits::<U2>(1i64 as u128).to_bools());
                        v.extend(foo.bin());
                        v
                    }
                    Self::C(_0) => {
                        let mut v = bitx_vec(&rhdl_bits::bits::<U2>(2i64 as u128).to_bools());
                        v.extend(_0.bin());
                        v
                    }
                })
            }
            fn discriminant(self) -> TypedBits {
                match self {
                    Self::A(_0) => rhdl_bits::bits::<U2>(0i64 as u128).typed_bits(),
                    Self::B { foo: _ } => rhdl_bits::bits::<U2>(1i64 as u128).typed_bits(),
                    Self::C(_0) => rhdl_bits::bits::<U2>(2i64 as u128).typed_bits(),
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
                    Self::C(_0) => Kind::make_tuple(vec![<b8 as Digital>::static_kind()]),
                }
            }
            fn dont_care() -> Self {
                use rand::Rng;
                match rand::thread_rng().gen_range(0..3) {
                    0 => Self::A(Default::default()),
                    1 => Self::B {
                        foo: Default::default(),
                    },
                    2 => Self::C(<b8 as Digital>::dont_care()),
                    _ => unreachable!(),
                }
            }
        }

        #[derive(Debug, Clone, PartialEq, Copy, Default)]
        struct Bar(b8, b8, bool);

        impl Digital for Bar {
            const BITS: usize = 17;
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
            fn static_trace_type() -> rhdl_trace_type::TraceType {
                crate::rtt::test::kind_to_trace(&Self::static_kind())
            }
            fn bin(self) -> Vec<BitX> {
                [self.0.bin(), self.1.bin(), self.2.bin()].concat()
            }
            fn dont_care() -> Self {
                Self(
                    <b8 as Digital>::dont_care(),
                    <b8 as Digital>::dont_care(),
                    <bool as Digital>::dont_care(),
                )
            }
        }

        #[derive(Debug, Clone, PartialEq, Copy, Default)]
        struct Foo {
            a: b8,
            b: b8,
            c: bool,
        }

        impl Digital for Foo {
            const BITS: usize = 17;
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
            fn static_trace_type() -> rhdl_trace_type::TraceType {
                crate::rtt::test::kind_to_trace(&Self::static_kind())
            }
            fn bin(self) -> Vec<BitX> {
                [self.a.bin(), self.b.bin(), self.c.bin()].concat()
            }
            fn dont_care() -> Self {
                Self {
                    a: <b8 as Digital>::dont_care(),
                    b: <b8 as Digital>::dont_care(),
                    c: <bool as Digital>::dont_care(),
                }
            }
        }

        assert_eq!(Baz::BITS, Baz::static_kind().bits());
        assert_eq!(Foo::BITS, Foo::static_kind().bits());
        assert_eq!(Bar::BITS, Bar::static_kind().bits());

        let a = b8(0x47).typed_bits();
        assert_eq!(format!("{:?}", a), "47_b8");
        let c = (b8(0x12), b8(0x80), false).typed_bits();
        assert_eq!(format!("{:?}", c), "(12_b8, 80_b8, false)");
        let b = (s32(-0x53)).typed_bits();
        assert_eq!(format!("{:?}", b), "-83_s32");
        let d = [b8(1), b8(3), b8(4)].typed_bits();
        assert_eq!(format!("{:?}", d), "[1_b8, 3_b8, 4_b8]");
        let e = Foo {
            a: b8(0x47),
            b: b8(0x80),
            c: true,
        }
        .typed_bits();
        assert_eq!(format!("{:?}", e), "Foo {a: 47_b8, b: 80_b8, c: true}");
        let e = Bar(b8(0x47), b8(0x80), true).typed_bits();
        assert_eq!(format!("{:?}", e), "Bar {0: 47_b8, 1: 80_b8, 2: true}");
        let d = [
            Bar(b8(0x47), b8(0x80), true),
            Bar(b8(0x42), b8(0x13), false),
        ]
        .typed_bits();
        assert_eq!(
            format!("{:?}", d),
            "[Bar {0: 47_b8, 1: 80_b8, 2: true}, Bar {0: 42_b8, 1: 13_b8, 2: false}]"
        );
        let h = Baz::A(Bar(b8(0x47), b8(0x80), true)).typed_bits();
        assert_eq!(
            format!("{:?}", h),
            "rhdl_core::types::typed_bits::tests::Baz::A(Bar {0: 47_b8, 1: 80_b8, 2: true})"
        );
    }

    #[test]
    fn test_result_promotion() {
        type MyR = std::result::Result<b4, b2>;
        let a = MyR::Ok(bits(2)).typed_bits();
        let b = b4(2).typed_bits().wrap_ok(&MyR::static_kind()).unwrap();
        assert_eq!(a, b);
        let c = MyR::Err(bits(2)).typed_bits();
        let d = b2(2).typed_bits().wrap_err(&MyR::static_kind()).unwrap();
        assert_eq!(c, d);
    }

    #[test]
    fn test_result_empty_error() {
        type MyR = std::result::Result<b4, ()>;
        let a = MyR::Ok(bits(2)).typed_bits();
        let b = b4(2).typed_bits().wrap_ok(&MyR::static_kind()).unwrap();
        assert_eq!(a, b);
        let c = MyR::Err(()).typed_bits();
        let d = ().typed_bits().wrap_err(&MyR::static_kind()).unwrap();
        assert_eq!(c, d);
    }

    #[test]
    fn test_result_empty_value() {
        type MyR = std::result::Result<(), b4>;
        let a = MyR::Ok(()).typed_bits();
        let b = ().typed_bits().wrap_ok(&MyR::static_kind()).unwrap();
        assert_eq!(a, b);
        let c = MyR::Err(bits(2)).typed_bits();
        let d = b4(2).typed_bits().wrap_err(&MyR::static_kind()).unwrap();
        assert_eq!(c, d);
    }

    #[test]
    fn test_result_both_empty() {
        type MyR = std::result::Result<(), ()>;
        let a = MyR::Ok(()).typed_bits();
        let b = ().typed_bits().wrap_ok(&MyR::static_kind()).unwrap();
        assert_eq!(a, b);
        let c = MyR::Err(()).typed_bits();
        let d = ().typed_bits().wrap_err(&MyR::static_kind()).unwrap();
        assert_eq!(c, d);
    }

    #[test]
    fn test_result_fails_with_mismatched_result_kind() {
        type MyR = std::result::Result<(), ()>;
        let b = b4(3);
        assert!(b.typed_bits().wrap_ok(&MyR::static_kind()).is_err());
        assert!(b.typed_bits().wrap_err(&MyR::static_kind()).is_err());
    }

    #[test]
    fn test_option_promotion() {
        type MyO = std::option::Option<b4>;
        let a = MyO::Some(bits(2)).typed_bits();
        let b = b4(2).typed_bits().wrap_some(&MyO::static_kind()).unwrap();
        assert_eq!(a, b);
        let c = MyO::None.typed_bits();
        let d = ().typed_bits().wrap_none(&MyO::static_kind()).unwrap();
        assert_eq!(c, d);
    }

    #[test]
    fn test_option_empty() {
        type MyO = std::option::Option<()>;
        let a = MyO::Some(()).typed_bits();
        let b = ().typed_bits().wrap_some(&MyO::static_kind()).unwrap();
        assert_eq!(a, b);
        let c = MyO::None.typed_bits();
        let d = ().typed_bits().wrap_none(&MyO::static_kind()).unwrap();
        assert_eq!(c, d);
    }

    #[test]
    fn test_option_fails_with_mismatched_option_kind() {
        type MyO = std::option::Option<()>;
        let b = b4(3);
        assert!(b.typed_bits().wrap_some(&MyO::static_kind()).is_err());
        assert!(b.typed_bits().wrap_none(&MyO::static_kind()).is_err());
    }
}
