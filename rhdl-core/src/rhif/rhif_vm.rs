use crate::compiler::ty::Ty;
use crate::kernel::ExternalKernelDef;
use crate::path::Path;
use crate::rhif::object::Object;
use crate::rhif::rhif_spec::{
    AluBinary, AluUnary, Array, Assign, Binary, Block, Case, CaseArgument, Cast, Discriminant,
    Enum, Exec, If, Index, Member, OpCode, Repeat, Slot, Struct, Tuple, Unary,
};
use crate::{ast::ast_impl::FunctionId, rhif::design::Design, TypedBits};
use crate::{Digital, KernelFnKind, Kind};

use anyhow::Result;

use anyhow::{anyhow, bail};

struct VMState<'a> {
    reg_stack: &'a mut [Option<TypedBits>],
    literals: &'a [TypedBits],
    blocks: &'a [Block],
    design: &'a Design,
    obj: &'a Object,
    early_return_signalled: bool,
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
                .ok_or(anyhow!("ICE Register {r} not found in register stack"))?
                .clone()
                .ok_or(anyhow!("ICE Register {r} is not initialized")),
            Slot::Empty => Ok(TypedBits::EMPTY),
        }
    }
    fn write(&mut self, slot: Slot, value: TypedBits) -> Result<()> {
        match slot {
            Slot::Literal(_) => bail!("ICE Cannot write to literal"),
            Slot::Register(r) => {
                self.reg_stack[r] = Some(value);
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
    fn resolve_dynamic_paths(&mut self, path: &Path) -> Result<Path> {
        let mut result = Path::default();
        for element in &path.elements {
            match element {
                crate::path::PathElement::DynamicIndex(slot) => {
                    let slot = self.read(*slot)?;
                    let ndx = slot.as_i64()?;
                    result = result.index(ndx as usize);
                }
                _ => result.elements.push(element.clone()),
            }
        }
        Ok(result)
    }
}

fn execute_block(block: &Block, state: &mut VMState) -> Result<()> {
    for op in &block.ops {
        if state.early_return_signalled {
            break;
        }
        match op {
            OpCode::Binary(Binary {
                op,
                lhs,
                arg1,
                arg2,
            }) => {
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
                    AluBinary::Lt => (arg1 < arg2).typed_bits(),
                    AluBinary::Le => (arg1 <= arg2).typed_bits(),
                    AluBinary::Gt => (arg1 > arg2).typed_bits(),
                    AluBinary::Ge => (arg1 >= arg2).typed_bits(),
                    _ => todo!(),
                };
                state.write(*lhs, result)?;
            }
            OpCode::Unary(Unary { op, lhs, arg1 }) => {
                let arg1 = state.read(*arg1)?;
                let result = match op {
                    AluUnary::Not => (!arg1)?,
                    AluUnary::Neg => (-arg1)?,
                    AluUnary::All => arg1.all(),
                    AluUnary::Any => arg1.any(),
                    AluUnary::Signed => arg1.as_signed()?,
                    AluUnary::Unsigned => arg1.as_unsigned()?,
                    AluUnary::Xor => arg1.xor(),
                };
                state.write(*lhs, result)?;
            }
            OpCode::Comment(_) => {}
            OpCode::If(If {
                lhs: _,
                cond,
                then_branch,
                else_branch,
            }) => {
                let cond = state.read(*cond)?;
                if cond.any().as_bool()? {
                    execute_block(&state.blocks[then_branch.0], state)?;
                } else {
                    execute_block(&state.blocks[else_branch.0], state)?;
                }
            }
            OpCode::Block(block_id) => {
                execute_block(&state.blocks[block_id.0], state)?;
            }
            OpCode::Index(Index { lhs, arg, path }) => {
                let arg = state.read(*arg)?;
                let path = state.resolve_dynamic_paths(path)?;
                let result = arg.path(&path)?;
                state.write(*lhs, result)?;
            }
            OpCode::Assign(Assign { lhs, rhs, path }) => {
                let rhs_val = state.read(*rhs)?;
                let path = state.resolve_dynamic_paths(path)?;
                if path.is_empty() {
                    state.write(*lhs, rhs_val)?;
                } else {
                    let lhs_val = state.read(*lhs)?;
                    let result = lhs_val.update(&path, rhs_val)?;
                    state.write(*lhs, result)?;
                }
            }
            OpCode::Tuple(Tuple { lhs, fields }) => {
                let fields = fields
                    .iter()
                    .map(|x| state.read(*x))
                    .collect::<Result<Vec<_>>>()?;
                let bits = fields
                    .iter()
                    .flat_map(|x| x.bits.iter().cloned())
                    .collect::<Vec<_>>();
                let kinds = fields.iter().map(|x| x.kind.clone()).collect::<Vec<_>>();
                let kind = Kind::make_tuple(kinds);
                let result = TypedBits { bits, kind };
                state.write(*lhs, result)?;
            }
            OpCode::Array(Array { lhs, elements }) => {
                let elements = elements
                    .iter()
                    .map(|x| state.read(*x))
                    .collect::<Result<Vec<_>>>()?;
                let bits = elements
                    .iter()
                    .flat_map(|x| x.bits.iter().cloned())
                    .collect::<Vec<_>>();
                let kind = Kind::make_array(elements[0].kind.clone(), elements.len());
                let result = TypedBits { bits, kind };
                state.write(*lhs, result)?;
            }
            OpCode::Struct(Struct {
                lhs,
                fields,
                rest,
                template,
            }) => {
                let mut result = if let Some(rest) = rest {
                    state.read(*rest)?
                } else {
                    template.clone()
                };
                for field in fields {
                    let value = state.read(field.value)?;
                    let path = match &field.member {
                        Member::Unnamed(ndx) => Path::default().index(*ndx as usize),
                        Member::Named(name) => Path::default().field(name),
                    };
                    result = result.update(&path, value)?;
                }
                state.write(*lhs, result)?;
            }
            OpCode::Enum(Enum {
                lhs,
                fields,
                template,
            }) => {
                let mut result = template.clone();
                for field in fields {
                    let base_path =
                        Path::default().payload_by_value(template.discriminant()?.as_i64()?);
                    let value = state.read(field.value)?;
                    let path = match &field.member {
                        Member::Unnamed(ndx) => base_path.index(*ndx as usize),
                        Member::Named(name) => base_path.field(name),
                    };
                    result = result.update(&path, value)?;
                }
                state.write(*lhs, result)?;
            }
            OpCode::Discriminant(Discriminant { lhs, arg }) => {
                let arg = state.read(*arg)?;
                let result = arg.discriminant()?;
                state.write(*lhs, result)?;
            }
            OpCode::Case(Case {
                discriminant,
                table,
            }) => {
                let discriminant = state.read(*discriminant)?;
                let block = table
                    .iter()
                    .find(|(disc, _)| match disc {
                        CaseArgument::Constant(disc) => discriminant == *disc,
                        CaseArgument::Wild => true,
                    })
                    .ok_or(anyhow!("ICE Case was not exhaustive"))?
                    .1;
                execute_block(&state.blocks[block.0], state)?;
            }
            OpCode::AsBits(Cast { lhs, arg, len }) => {
                let arg = state.read(*arg)?;
                let result = arg.unsigned_cast(*len)?;
                state.write(*lhs, result)?;
            }
            OpCode::AsSigned(Cast { lhs, arg, len }) => {
                let arg = state.read(*arg)?;
                let result = arg.signed_cast(*len)?;
                state.write(*lhs, result)?;
            }
            OpCode::Exec(Exec { lhs, id, args }) => {
                let args = args
                    .iter()
                    .map(|x| state.read(*x))
                    .collect::<Result<Vec<_>>>()?;
                let func = &state.obj.externals[id.0];
                let result = match &func.code {
                    KernelFnKind::Kernel(kernel) => execute(state.design, kernel.fn_id, args)?,
                    KernelFnKind::Extern(ExternalKernelDef {
                        name,
                        body: _,
                        vm_stub,
                    }) => {
                        if let Some(stub) = vm_stub {
                            stub(&args)?
                        } else {
                            bail!("No VM stub for {name}")
                        }
                    }
                    _ => {
                        todo!("No support for non-AST kernels {func:?}")
                    }
                };
                state.write(*lhs, result)?;
            }
            OpCode::Repeat(Repeat { lhs, value, len }) => {
                let value = state.read(*value)?;
                let result = value.repeat(*len);
                state.write(*lhs, result)?;
            }
            OpCode::Return => {
                state.early_return_signalled = true;
                break;
            }
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
    let mut reg_stack = vec![None; max_reg + 1];
    // Copy the arguments into the appropriate registers
    for (ndx, arg) in arguments.into_iter().enumerate() {
        if let Slot::Register(r) = obj.arguments[ndx] {
            reg_stack[r] = Some(arg);
        }
    }
    let block = obj
        .blocks
        .get(obj.main_block.0)
        .ok_or(anyhow!("main block not found"))?;
    let mut state = VMState {
        reg_stack: &mut reg_stack,
        literals: &obj.literals,
        blocks: &obj.blocks,
        design,
        obj,
        early_return_signalled: false,
    };
    execute_block(block, &mut state)?;
    reg_stack
        .get(obj.return_slot.reg()?)
        .cloned()
        .ok_or(anyhow!("return slot not found"))?
        .ok_or(anyhow!("ICE return slot is not initialized"))
}

// Given a set of arguments in the form of TypedBits, execute the function described by a Design
// and then return the result as a TypedBits.
pub fn execute_function(design: &Design, arguments: Vec<TypedBits>) -> Result<TypedBits> {
    execute(design, design.top, arguments)
}
