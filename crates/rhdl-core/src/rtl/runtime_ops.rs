use crate::rhdl_core::{RHDLError, TypedBits};

use super::spec::{AluBinary, AluUnary};

pub fn unary(op: AluUnary, arg1: TypedBits) -> Result<TypedBits, RHDLError> {
    let op = op.into();
    crate::rhdl_core::rhif::runtime_ops::unary(op, arg1)
}

pub fn binary(op: AluBinary, arg1: TypedBits, arg2: TypedBits) -> Result<TypedBits, RHDLError> {
    let op = op.into();
    crate::rhdl_core::rhif::runtime_ops::binary(op, arg1, arg2)
}
