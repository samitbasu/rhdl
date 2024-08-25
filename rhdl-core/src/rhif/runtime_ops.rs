use crate::{Digital, Kind, RHDLError, TypedBits};

use super::spec::{AluBinary, AluUnary};

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
        AluBinary::Mul => todo!(),
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
    let kinds = fields.iter().map(|x| x.kind.clone()).collect::<Vec<_>>();
    let kind = Kind::make_tuple(kinds);
    TypedBits { bits, kind }
}

pub fn array(elements: &[TypedBits]) -> TypedBits {
    let bits = elements
        .iter()
        .flat_map(|x| x.bits.iter().cloned())
        .collect::<Vec<_>>();
    let kind = Kind::make_array(elements[0].kind.clone(), elements.len());
    TypedBits { bits, kind }
}
