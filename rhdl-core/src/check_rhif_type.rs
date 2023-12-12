// Check a RHIF object for type correctness.

use crate::{
    rhif::{Object, OpCode, Slot},
    ty::{self, ty_array, ty_array_base, ty_as_ref, Ty},
    TypedBits,
};
use anyhow::anyhow;
use anyhow::Result;

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
            match op {
                OpCode::Binary {
                    op,
                    lhs,
                    arg1,
                    arg2,
                } => {
                    eq_types(slot_type(lhs)?, slot_type(arg1)?)?;
                    eq_types(slot_type(lhs)?, slot_type(arg2)?)?;
                }
                OpCode::Unary { op, lhs, arg1 } => {
                    eq_types(slot_type(lhs)?, slot_type(arg1)?)?;
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
