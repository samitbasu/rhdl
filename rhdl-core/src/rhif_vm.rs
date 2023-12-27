use crate::rhif::{AluBinary, Block, OpCode, Slot};
use crate::ty::Ty;
use crate::Digital;
use crate::{ast::FunctionId, design::Design, object::Object, TypedBits};

use anyhow::Result;

use anyhow::{anyhow, bail};
struct VMState<'a> {
    reg_stack: &'a mut [TypedBits],
    literals: &'a [TypedBits],
}

impl<'a> VMState<'a> {
    fn read(&self, slot: Slot) -> Result<TypedBits> {
        match slot {
            Slot::Literal(l) => self
                .literals
                .get(l)
                .cloned()
                .ok_or(anyhow!("ICE Literal {l} not found in object")),
            Slot::Register(r) => self
                .reg_stack
                .get(r)
                .cloned()
                .ok_or(anyhow!("ICE Register {r} not found in register stack")),
            Slot::Empty => Ok(TypedBits::EMPTY),
        }
    }
    fn write(&mut self, slot: Slot, value: TypedBits) -> Result<()> {
        match slot {
            Slot::Literal(_) => bail!("ICE Cannot write to literal"),
            Slot::Register(r) => {
                self.reg_stack[r] = value;
                Ok(())
            }
            Slot::Empty => {
                if value.kind.is_empty() {
                    Ok(())
                } else {
                    bail!("ICE Cannot write non-empty value to empty slot")
                }
            }
        }
    }
}

fn execute_block(block: &Block, state: &mut VMState) -> Result<()> {
    for op in &block.ops {
        match op {
            OpCode::Copy { lhs, rhs } => {
                state.write(*lhs, state.read(*rhs)?.clone())?;
            }
            OpCode::Binary {
                op,
                lhs,
                arg1,
                arg2,
            } => {
                let arg1 = state.read(*arg1)?;
                let arg2 = state.read(*arg2)?;
                let result = match op {
                    AluBinary::Add => (arg1 + arg2)?,
                    AluBinary::Sub => (arg1 - arg2)?,
                    AluBinary::BitXor => (arg1 ^ arg2)?,
                    AluBinary::BitAnd => (arg1 & arg2)?,
                    AluBinary::BitOr => (arg1 | arg2)?,
                    AluBinary::Eq => (arg1 == arg2).typed_bits(),
                    AluBinary::Ne => (arg1 != arg2).typed_bits(),
                    AluBinary::Shl => (arg1 << arg2)?,
                    AluBinary::Shr => (arg1 >> arg2)?,
                    //                    AluBinary::Ge => (arg1 >= arg2).typed_bits(),
                    _ => todo!(),
                };
                state.write(*lhs, result)?;
            }
            OpCode::Comment(_) => {}
            _ => todo!(),
        }
    }
    Ok(())
}

fn execute(design: &Design, fn_id: FunctionId, arguments: Vec<TypedBits>) -> Result<TypedBits> {
    // Load the object for this function
    let obj = design
        .objects
        .get(&fn_id)
        .ok_or(anyhow::anyhow!("Function {fn_id} not found"))?;
    if obj.arguments.len() != arguments.len() {
        bail!(
            "Function {fn_id} expected {expected} arguments, got {got}",
            fn_id = fn_id,
            expected = obj.arguments.len(),
            got = arguments.len()
        );
    }
    for (ndx, arg) in arguments.iter().enumerate() {
        let arg_ty: Ty = arg.kind.clone().into();
        let obj_ty = obj
            .ty
            .get(&obj.arguments[ndx])
            .ok_or(anyhow!("ICE argument {ndx} type not found in object"))?;
        if obj_ty != &arg_ty {
            bail!(
                "Function {fn_id} argument {ndx} expected {expected}, got {got}",
                fn_id = fn_id,
                ndx = ndx,
                expected = obj.ty.get(&obj.arguments[ndx]).unwrap(),
                got = arg_ty
            );
        }
    }
    // Allocate registers for the function call.
    let max_reg = obj.reg_count() + 1;
    let mut reg_stack = vec![TypedBits::EMPTY; max_reg + 1];
    // Copy the arguments into the appropriate registers
    for (ndx, arg) in arguments.into_iter().enumerate() {
        reg_stack[obj.arguments[ndx].reg()?] = arg;
    }
    let block = obj
        .blocks
        .get(obj.main_block.0)
        .ok_or(anyhow!("main block not found"))?;
    let mut state = VMState {
        reg_stack: &mut reg_stack,
        literals: &obj.literals,
    };
    execute_block(block, &mut state)?;
    reg_stack
        .get(obj.return_slot.reg()?)
        .cloned()
        .ok_or(anyhow!("return slot not found"))
}

// Given a set of arguments in the form of TypedBits, execute the function described by a Design
// and then return the result as a TypedBits.
pub fn execute_function(design: &Design, arguments: Vec<TypedBits>) -> Result<TypedBits> {
    execute(design, design.top, arguments)
}
