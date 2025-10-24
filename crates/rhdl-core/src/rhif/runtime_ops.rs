use num_bigint::BigInt;

use crate::bitx::dyn_bit_manip::{from_bigint, from_biguint, to_bigint, to_biguint};
use crate::error::rhdl_error;
use crate::types::error::DynamicTypeError;
use crate::{Digital, Kind, RHDLError, TypedBits};

use super::spec::{AluBinary, AluUnary};

// Oh, the horrors.

fn mul(a: TypedBits, b: TypedBits) -> Result<TypedBits, RHDLError> {
    if a.kind().is_signed() ^ b.kind().is_signed() {
        return Err(rhdl_error(
            DynamicTypeError::BinaryOperationRequiresCompatibleType {
                lhs: a.kind(),
                rhs: b.kind(),
            },
        ));
    }
    if a.kind().is_signed() {
        let a_bi = to_bigint(a.bits()).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: a.clone() })
        })?;
        let b_bi = to_bigint(b.bits()).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: b.clone() })
        })?;
        let result = a_bi * b_bi;
        Ok(TypedBits::new(
            from_bigint(&result, a.len()).into(),
            Kind::Signed(a.len()),
        ))
    } else {
        let a_bi = to_biguint(a.bits()).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: a.clone() })
        })?;
        let b_bi = to_biguint(b.bits()).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: b.clone() })
        })?;
        let result = a_bi * b_bi;
        Ok(TypedBits::new(
            from_biguint(&result, a.len()),
            Kind::Bits(a.len()),
        ))
    }
}

fn xsub(a: TypedBits, b: TypedBits) -> Result<TypedBits, RHDLError> {
    let size_fn = |a: usize, b| a.max(b) + 1;
    if a.kind().is_signed() ^ b.kind().is_signed() {
        return Err(rhdl_error(
            DynamicTypeError::BinaryOperationRequiresCompatibleType {
                lhs: a.kind(),
                rhs: b.kind(),
            },
        ));
    }
    if a.kind().is_signed() {
        let a_bi = to_bigint(a.bits()).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: a.clone() })
        })?;
        let b_bi = to_bigint(b.bits()).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: b.clone() })
        })?;
        let result = a_bi - b_bi;
        Ok(TypedBits::new(
            from_bigint(&result, size_fn(a.len(), b.len())).into(),
            Kind::Signed(size_fn(a.len(), b.len())),
        ))
    } else {
        let a_bi = to_biguint(a.bits()).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: a.clone() })
        })?;
        let b_bi = to_biguint(b.bits()).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: b.clone() })
        })?;
        let a_bi: BigInt = a_bi.into();
        let b_bi: BigInt = b_bi.into();
        let result = a_bi - b_bi;
        Ok(TypedBits::new(
            from_bigint(&result, size_fn(a.len(), b.len())).into(),
            Kind::Signed(size_fn(a.len(), b.len())),
        ))
    }
}

fn xadd(a: TypedBits, b: TypedBits) -> Result<TypedBits, RHDLError> {
    let size_fn = |a: usize, b| a.max(b) + 1;
    if a.kind().is_signed() ^ b.kind().is_signed() {
        return Err(rhdl_error(
            DynamicTypeError::BinaryOperationRequiresCompatibleType {
                lhs: a.kind(),
                rhs: b.kind(),
            },
        ));
    }
    if a.kind().is_signed() {
        let a_bi = to_bigint(a.bits()).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: a.clone() })
        })?;
        let b_bi = to_bigint(b.bits()).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: b.clone() })
        })?;
        let result = a_bi + b_bi;
        Ok(TypedBits::new(
            from_bigint(&result, size_fn(a.len(), b.len())).into(),
            Kind::Signed(size_fn(a.len(), b.len())),
        ))
    } else {
        let a_bi = to_biguint(a.bits()).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: a.clone() })
        })?;
        let b_bi = to_biguint(b.bits()).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: b.clone() })
        })?;
        let result = a_bi + b_bi;
        Ok(TypedBits::new(
            from_biguint(&result, size_fn(a.len(), b.len())),
            Kind::Bits(size_fn(a.len(), b.len())),
        ))
    }
}

fn xmul(a: TypedBits, b: TypedBits) -> Result<TypedBits, RHDLError> {
    if a.kind().is_signed() ^ b.kind().is_signed() {
        return Err(rhdl_error(
            DynamicTypeError::BinaryOperationRequiresCompatibleType {
                lhs: a.kind(),
                rhs: b.kind(),
            },
        ));
    }
    if a.kind().is_signed() {
        let a_bi = to_bigint(a.bits()).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: a.clone() })
        })?;
        let b_bi = to_bigint(b.bits()).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: b.clone() })
        })?;
        let result = a_bi * b_bi;
        Ok(TypedBits::new(
            from_bigint(&result, a.len() + b.len()).into(),
            Kind::Signed(a.len() + b.len()),
        ))
    } else {
        let a_bi = to_biguint(a.bits()).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: a.clone() })
        })?;
        let b_bi = to_biguint(b.bits()).ok_or_else(|| {
            rhdl_error(DynamicTypeError::CannotConvertUninitToInt { value: b.clone() })
        })?;
        let result = a_bi * b_bi;
        Ok(TypedBits::new(
            from_biguint(&result, a.len() + b.len()),
            Kind::Bits(a.len() + b.len()),
        ))
    }
}

pub fn binary(
    op: crate::rhif::spec::AluBinary,
    arg1: TypedBits,
    arg2: TypedBits,
) -> Result<TypedBits, RHDLError> {
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
        AluBinary::XAdd => xadd(arg1, arg2),
        AluBinary::XSub => xsub(arg1, arg2),
        AluBinary::XMul => xmul(arg1, arg2),
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
        AluUnary::XExt(diff) => arg1.xext(diff),
        AluUnary::XShl(diff) => arg1.xshl(diff),
        AluUnary::XShr(diff) => arg1.xshr(diff),
        AluUnary::XNeg => {
            let arg1 = arg1.xext(1)?;
            let arg1 = if arg1.kind().is_unsigned() {
                arg1.as_signed()?
            } else {
                arg1
            };
            -arg1
        }
        AluUnary::XSgn => {
            let arg1 = arg1.xext(1)?;
            arg1.as_signed()
        }
    }
}

pub fn tuple(fields: &[TypedBits]) -> TypedBits {
    let bits = fields
        .iter()
        .flat_map(|x| x.iter().cloned())
        .collect::<Vec<_>>();
    let kinds = fields.iter().map(|x| x.kind()).collect::<Vec<_>>();
    let kind = Kind::make_tuple(kinds.into());
    TypedBits::new(bits, kind)
}

pub fn array(elements: &[TypedBits]) -> TypedBits {
    let bits = elements
        .iter()
        .flat_map(|x| x.iter().cloned())
        .collect::<Vec<_>>();
    let kind = Kind::make_array(elements[0].kind(), elements.len());
    TypedBits::new(bits, kind)
}
