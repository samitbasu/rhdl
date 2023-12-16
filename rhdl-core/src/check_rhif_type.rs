// Check a RHIF object for type correctness.

use crate::{
    rhif::{AluBinary, AluUnary, Object, OpCode, Slot},
    ty::{self, ty_array, ty_array_base, ty_as_ref, ty_bool, Bits, Ty},
    TypedBits,
};
use anyhow::Result;
use anyhow::{anyhow, bail};

pub fn check_type_correctness(obj: &Object) -> Result<()> {
    let slot_type = |slot| -> Result<Ty> {
        obj.ty
            .get(slot)
            .cloned()
            .ok_or(anyhow!("slot {:?} not found", slot))
    };
    let eq_types = |a, b| -> Result<()> {
        if a == b {
            Ok(())
        } else {
            Err(anyhow!("type mismatch: {:?} != {:?}", a, b))
        }
    };
    for block in &obj.blocks {
        for op in &block.ops {
            eprintln!("op: {:?}", op);
            match op {
                OpCode::Binary {
                    op:
                        AluBinary::Add
                        | AluBinary::Sub
                        | AluBinary::Mul
                        | AluBinary::BitAnd
                        | AluBinary::BitOr
                        | AluBinary::BitXor
                        | AluBinary::Shl
                        | AluBinary::Shr,
                    lhs,
                    arg1,
                    arg2,
                } => {
                    eq_types(slot_type(lhs)?, slot_type(arg1)?)?;
                    eq_types(slot_type(lhs)?, slot_type(arg2)?)?;
                }
                OpCode::Binary {
                    op: AluBinary::And | AluBinary::Or,
                    lhs,
                    arg1,
                    arg2,
                } => {
                    eq_types(slot_type(lhs)?, slot_type(arg1)?)?;
                    eq_types(slot_type(lhs)?, slot_type(arg2)?)?;
                    eq_types(slot_type(lhs)?, ty_bool())?;
                }
                OpCode::Binary {
                    op:
                        AluBinary::Eq
                        | AluBinary::Ge
                        | AluBinary::Gt
                        | AluBinary::Le
                        | AluBinary::Lt
                        | AluBinary::Ne,
                    lhs,
                    arg1,
                    arg2,
                } => {
                    eq_types(slot_type(arg1)?, slot_type(arg2)?)?;
                    eq_types(slot_type(lhs)?, ty_bool())?;
                }
                OpCode::Unary {
                    op: AluUnary::Not | AluUnary::Neg,
                    lhs,
                    arg1,
                } => {
                    eq_types(slot_type(lhs)?, slot_type(arg1)?)?;
                }
                OpCode::Unary {
                    op: AluUnary::All | AluUnary::Any | AluUnary::Xor,
                    lhs,
                    arg1,
                } => {
                    eq_types(slot_type(lhs)?, ty_bool())?;
                }
                OpCode::Unary {
                    op: AluUnary::Signed,
                    lhs,
                    arg1,
                } => {
                    let arg1_ty = slot_type(arg1)?;
                    let Ty::Const(Bits::Unsigned(x)) = arg1_ty else {
                        bail!("signed operator requires unsigned argument")
                    };
                    eq_types(slot_type(lhs)?, ty::ty_signed(x))?;
                }
                OpCode::Unary {
                    op: AluUnary::Unsigned,
                    lhs,
                    arg1,
                } => {
                    let arg1_ty = slot_type(arg1)?;
                    let Ty::Const(Bits::Signed(x)) = arg1_ty else {
                        bail!("unsigned operator requires signed argument")
                    };
                    eq_types(slot_type(lhs)?, ty::ty_bits(x))?;
                }
                OpCode::Array { lhs, elements } => eq_types(
                    slot_type(lhs)?,
                    ty_array(slot_type(&elements[0])?, elements.len()),
                )?,
                OpCode::If {
                    lhs,
                    cond,
                    then_branch,
                    else_branch,
                } => {
                    eq_types(slot_type(cond)?, ty::ty_bool())?;
                }
                OpCode::Index { lhs, arg, index } => {
                    eq_types(slot_type(lhs)?, ty_array_base(&slot_type(arg)?)?)?;
                }
                OpCode::Ref { lhs, arg } => {
                    eq_types(slot_type(lhs)?, ty_as_ref(slot_type(arg)?))?;
                }
                OpCode::Assign { lhs, rhs } => {
                    eq_types(slot_type(lhs)?, ty_as_ref(slot_type(rhs)?))?;
                }
                _ => {}
            }
        }
    }
    Ok(())
}
