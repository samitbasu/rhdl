use crate::bitx::dyn_bit_manip::{from_bigint, from_biguint, to_bigint, to_biguint};
use crate::error::rhdl_error;
use crate::types::error::DynamicTypeError;
use crate::{Digital, Kind, RHDLError, TypedBits};

use super::spec::{AluBinary, AluUnary};

// Oh, the horrors.
fn mul(a: TypedBits, b: TypedBits) -> Result<TypedBits, RHDLError> {
    if a.kind.is_signed() ^ b.kind.is_signed() {
        return Err(rhdl_error(
            DynamicTypeError::BinaryOperationRequiresCompatibleType {
                lhs: a.kind,
                rhs: b.kind,
            },
        ));
    }
    if a.kind.is_signed() {
        let a_bi = to_bigint(&a.bits).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: a.clone() })
        })?;
        let b_bi = to_bigint(&b.bits).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: b.clone() })
        })?;
        let result = a_bi * b_bi;
        Ok(TypedBits {
            bits: from_bigint(&result, a.bits.len() + b.bits.len()),
            kind: Kind::Signed(a.bits.len() + b.bits.len()),
        })
    } else {
        let a_bi = to_biguint(&a.bits).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: a.clone() })
        })?;
        let b_bi = to_biguint(&b.bits).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: b.clone() })
        })?;
        let result = a_bi * b_bi;
        Ok(TypedBits {
            bits: from_biguint(&result, a.bits.len() + b.bits.len()),
            kind: Kind::Bits(a.bits.len() + b.bits.len()),
        })
    }
}

fn mul_rtl(a: TypedBits, b: TypedBits) -> Result<TypedBits, RHDLError> {
    if a.kind.is_signed() ^ b.kind.is_signed() {
        return Err(rhdl_error(
            DynamicTypeError::BinaryOperationRequiresCompatibleType {
                lhs: a.kind,
                rhs: b.kind,
            },
        ));
    }
    let c_bits = a.bits.len().min(b.bits.len());
    if a.kind.is_signed() {
        let a_bi = to_bigint(&a.bits).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: a.clone() })
        })?;
        let b_bi = to_bigint(&b.bits).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: b.clone() })
        })?;
        let result = a_bi * b_bi;
        Ok(TypedBits {
            bits: from_bigint(&result, c_bits),
            kind: Kind::Signed(c_bits),
        })
    } else {
        let a_bi = to_biguint(&a.bits).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: a.clone() })
        })?;
        let b_bi = to_biguint(&b.bits).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: b.clone() })
        })?;
        let result = a_bi * b_bi;
        Ok(TypedBits {
            bits: from_biguint(&result, c_bits),
            kind: Kind::Bits(c_bits),
        })
    }
}

pub fn binary(op: AluBinary, arg1: TypedBits, arg2: TypedBits) -> Result<TypedBits, RHDLError> {
    match op {
        AluBinary::Add => arg1 + arg2,
        AluBinary::Sub => arg1 - arg2,
        AluBinary::BitXor => arg1 ^ arg2,
        AluBinary::BitAnd => arg1 & arg2,
        AluBinary::BitOr => arg1 | arg2,
        AluBinary::Eq => Ok((arg1 == arg2).typed_bits()),
        AluBinary::Ne => Ok((arg1 != arg2).typed_bits()),
        AluBinary::Shl => arg1 << arg2,
        AluBinary::Shr => arg1 >> arg2,
        AluBinary::Lt => Ok((arg1 < arg2).typed_bits()),
        AluBinary::Le => Ok((arg1 <= arg2).typed_bits()),
        AluBinary::Gt => Ok((arg1 > arg2).typed_bits()),
        AluBinary::Ge => Ok((arg1 >= arg2).typed_bits()),
        AluBinary::Mul => mul(arg1, arg2),
    }
}

pub fn binary_rtl(op: AluBinary, arg1: TypedBits, arg2: TypedBits) -> Result<TypedBits, RHDLError> {
    match op {
        AluBinary::Mul => mul_rtl(arg1, arg2),
        _ => binary(op, arg1, arg2),
    }
}

pub fn unary(op: AluUnary, arg1: TypedBits) -> Result<TypedBits, RHDLError> {
    match op {
        AluUnary::Not => !arg1,
        AluUnary::Neg => -arg1,
        AluUnary::All => Ok(arg1.all()),
        AluUnary::Any => Ok(arg1.any()),
        AluUnary::Signed => arg1.as_signed(),
        AluUnary::Unsigned => arg1.as_unsigned(),
        AluUnary::Xor => Ok(arg1.xor()),
        AluUnary::Val => Ok(arg1.val()),
    }
}

pub fn tuple(fields: &[TypedBits]) -> TypedBits {
    let bits = fields
        .iter()
        .flat_map(|x| x.bits.iter().cloned())
        .collect::<Vec<_>>();
    let kinds = fields.iter().map(|x| x.kind).collect::<Vec<_>>();
    let kind = Kind::make_tuple(kinds);
    TypedBits { bits, kind }
}

pub fn array(elements: &[TypedBits]) -> TypedBits {
    let bits = elements
        .iter()
        .flat_map(|x| x.bits.iter().cloned())
        .collect::<Vec<_>>();
    let kind = Kind::make_array(elements[0].kind, elements.len());
    TypedBits { bits, kind }
}
