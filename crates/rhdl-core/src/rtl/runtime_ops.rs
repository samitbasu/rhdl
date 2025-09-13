use crate::{RHDLError, TypedBits};

use super::spec::{AluBinary, AluUnary};

pub fn unary(op: AluUnary, arg1: TypedBits) -> Result<TypedBits, RHDLError> {
    let op = op.into();
    crate::rhif::runtime_ops::unary(op, arg1)
}

pub fn binary(op: AluBinary, arg1: TypedBits, arg2: TypedBits) -> Result<TypedBits, RHDLError> {
    let op = op.into();
    crate::rhif::runtime_ops::binary(op, arg1, arg2)
}
