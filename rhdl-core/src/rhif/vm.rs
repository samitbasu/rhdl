use std::collections::BTreeMap;

use crate::ast::ast_impl::{NodeId, WrapOp};
use crate::compiler::mir::error::{RHDLCompileError, ICE};
use crate::error::rhdl_error;
use crate::rhif::object::Object;
use crate::rhif::spec::{
    Array, Assign, Binary, Case, CaseArgument, Cast, Enum, Exec, Index, Member, OpCode, Repeat,
    Slot, Struct, Tuple, Unary,
};
use crate::types::path::Path;
use crate::TypedBits;
use crate::{Kind, RHDLError};

use super::object::LocatedOpCode;
use super::runtime_ops::{array, binary, tuple, unary};
use super::spec::{LiteralId, Retime, Select, Splice, Wrap};

type Result<T> = std::result::Result<T, RHDLError>;

struct VMState<'a> {
    reg_stack: &'a mut [Option<TypedBits>],
    literals: &'a BTreeMap<LiteralId, TypedBits>,
    obj: &'a Object,
}

impl<'a> VMState<'a> {
    fn raise_ice(&self, cause: ICE, id: NodeId) -> RHDLError {
        rhdl_error(RHDLCompileError {
            cause,
            src: self.obj.symbols.source.source.clone(),
            err_span: self.obj.symbols.node_span(id).into(),
        })
    }
    fn read(&self, slot: Slot, id: NodeId) -> Result<TypedBits> {
        match slot {
            Slot::Literal(l) => Ok(self.literals[&l].clone()),
            Slot::Register(r) => self.reg_stack[r.0]
                .clone()
                .ok_or(self.raise_ice(ICE::UninitializedRegister { r }, id)),
            Slot::Empty => Ok(TypedBits::EMPTY),
        }
    }
    fn write(&mut self, slot: Slot, value: TypedBits, id: NodeId) -> Result<()> {
        match slot {
            Slot::Literal(ndx) => Err(self.raise_ice(ICE::CannotWriteToRHIFLiteral { ndx }, id)),
            Slot::Register(r) => {
                self.reg_stack[r.0] = Some(value);
                Ok(())
            }
            Slot::Empty => {
                if value.kind.is_empty() {
                    Ok(())
                } else {
                    Err(self.raise_ice(ICE::CannotWriteNonEmptyValueToEmptySlot, id))
                }
            }
        }
    }
    fn resolve_dynamic_paths(&mut self, path: &Path, id: NodeId) -> Result<Path> {
        let mut result = Path::default();
        for element in &path.elements {
            match element {
                crate::types::path::PathElement::DynamicIndex(slot) => {
                    let slot = self.read(*slot, id)?;
                    let ndx = slot.as_i64()?;
                    result = result.index(ndx as usize);
                }
                _ => result.elements.push(element.clone()),
            }
        }
        Ok(result)
    }
}

fn execute_block(ops: &[LocatedOpCode], state: &mut VMState) -> Result<()> {
    for lop in ops {
        let op = &lop.op;
        let id = lop.id;
        match op {
            OpCode::Noop => {}
            OpCode::Binary(Binary {
                op,
                lhs,
                arg1,
                arg2,
            }) => {
                let arg1 = state.read(*arg1, id)?;
                let arg2 = state.read(*arg2, id)?;
                let result = binary(*op, arg1, arg2)?;
                state.write(*lhs, result, id)?;
            }
            OpCode::Unary(Unary { op, lhs, arg1 }) => {
                let arg1 = state.read(*arg1, id)?;
                let result = unary(*op, arg1)?;
                state.write(*lhs, result, id)?;
            }
            OpCode::Comment(_) => {}
            OpCode::Select(Select {
                lhs,
                cond,
                true_value,
                false_value,
            }) => {
                let cond = state.read(*cond, id)?;
                let true_value = state.read(*true_value, id)?;
                let false_value = state.read(*false_value, id)?;
                if cond.any().as_bool()? {
                    state.write(*lhs, true_value, id)?;
                } else {
                    state.write(*lhs, false_value, id)?;
                }
            }
            OpCode::Index(Index { lhs, arg, path }) => {
                let arg = state.read(*arg, id)?;
                let path = state.resolve_dynamic_paths(path, id)?;
                let result = arg.path(&path)?;
                state.write(*lhs, result, id)?;
            }
            OpCode::Splice(Splice {
                lhs,
                orig: rhs,
                path,
                subst: arg,
            }) => {
                let rhs_val = state.read(*rhs, id)?;
                let path = state.resolve_dynamic_paths(path, id)?;
                let arg_val = state.read(*arg, id)?;
                let result = rhs_val.splice(&path, arg_val)?;
                state.write(*lhs, result, id)?;
            }
            OpCode::Assign(Assign { lhs, rhs }) => {
                state.write(*lhs, state.read(*rhs, id)?, id)?;
            }
            OpCode::Tuple(Tuple { lhs, fields }) => {
                let fields = fields
                    .iter()
                    .map(|x| state.read(*x, id))
                    .collect::<Result<Vec<_>>>()?;
                let result = tuple(&fields);
                state.write(*lhs, result, id)?;
            }
            OpCode::Array(Array { lhs, elements }) => {
                let elements = elements
                    .iter()
                    .map(|x| state.read(*x, id))
                    .collect::<Result<Vec<_>>>()?;
                let result = array(&elements);
                state.write(*lhs, result, id)?;
            }
            OpCode::Struct(Struct {
                lhs,
                fields,
                rest,
                template,
            }) => {
                let mut result = if let Some(rest) = rest {
                    state.read(*rest, id)?
                } else {
                    template.clone()
                };
                for field in fields {
                    let value = state.read(field.value, id)?;
                    let path = match &field.member {
                        Member::Unnamed(ndx) => Path::default().tuple_index(*ndx as usize),
                        Member::Named(name) => Path::default().field(name),
                    };
                    result = result.splice(&path, value)?;
                }
                state.write(*lhs, result, id)?;
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
                    let value = state.read(field.value, id)?;
                    let path = match &field.member {
                        Member::Unnamed(ndx) => base_path.tuple_index(*ndx as usize),
                        Member::Named(name) => base_path.field(name),
                    };
                    result = result.splice(&path, value)?;
                }
                state.write(*lhs, result, id)?;
            }
            OpCode::Case(Case {
                lhs,
                discriminant,
                table,
            }) => {
                let discriminant = state.read(*discriminant, id)?;
                let arm = table
                    .iter()
                    .find(|(disc, _)| match disc {
                        CaseArgument::Slot(disc) => discriminant == state.read(*disc, id).unwrap(),
                        CaseArgument::Wild => true,
                    })
                    .ok_or(state.raise_ice(ICE::NoMatchingArm { discriminant }, id))?
                    .1;
                let arm = state.read(arm, id)?;
                state.write(*lhs, arm, id)?;
            }
            OpCode::AsBits(Cast { lhs, arg, len }) => {
                let arg = state.read(*arg, id)?;
                let len = len.ok_or(state.raise_ice(ICE::BitCastMissingRequiredLength, id))?;
                let result = arg.unsigned_cast(len)?;
                state.write(*lhs, result, id)?;
            }
            OpCode::AsSigned(Cast { lhs, arg, len }) => {
                let arg = state.read(*arg, id)?;
                let len = len.ok_or(state.raise_ice(ICE::BitCastMissingRequiredLength, id))?;
                let result = arg.signed_cast(len)?;
                state.write(*lhs, result, id)?;
            }
            OpCode::Resize(Cast { lhs, arg, len }) => {
                let arg = state.read(*arg, id)?;
                let len = len.ok_or(state.raise_ice(ICE::BitCastMissingRequiredLength, id))?;
                let result = arg.resize(len)?;
                state.write(*lhs, result, id)?;
            }
            OpCode::Retime(Retime { lhs, arg, color }) => {
                let mut arg = state.read(*arg, id)?;
                if let Some(color) = color {
                    arg.kind = Kind::make_signal(arg.kind, *color);
                }
                state.write(*lhs, arg, id)?;
            }
            OpCode::Wrap(Wrap { op, lhs, arg, kind }) => {
                let arg = state.read(*arg, id)?;
                let Some(kind) = kind else {
                    return Err(state.raise_ice(ICE::WrapMissingKind, id));
                };
                let arg = match op {
                    WrapOp::Ok => arg.wrap_ok(kind),
                    WrapOp::Err => arg.wrap_err(kind),
                    WrapOp::Some => arg.wrap_some(kind),
                    WrapOp::None => arg.wrap_none(kind),
                };
                state.write(*lhs, arg?, id)?;
            }
            OpCode::Exec(Exec {
                lhs,
                id: f_id,
                args,
            }) => {
                let args = args
                    .iter()
                    .map(|x| state.read(*x, id))
                    .collect::<Result<Vec<_>>>()?;
                let func = &state.obj.externals[f_id];
                let result = execute(func, args)?;
                state.write(*lhs, result, id)?;
            }
            OpCode::Repeat(Repeat { lhs, value, len }) => {
                let value = state.read(*value, id)?;
                let len = *len as usize;
                let result = value.repeat(len);
                state.write(*lhs, result, id)?;
            }
        }
    }
    Ok(())
}

pub fn execute(obj: &Object, arguments: Vec<TypedBits>) -> Result<TypedBits> {
    // Load the object for this function
    if obj.arguments.len() != arguments.len() {
        return Err(rhdl_error(RHDLCompileError {
            cause: ICE::ArgumentCountMismatchOnCall,
            src: obj.symbols.source.source.clone(),
            err_span: obj.symbols.node_span(obj.ops[0].id).into(),
        }));
    }
    for (ndx, arg) in arguments.iter().enumerate() {
        let arg_kind = &arg.kind;
        let obj_kind = &obj.kind[&obj.arguments[ndx]];
        if obj_kind != arg_kind {
            return Err(rhdl_error(RHDLCompileError {
                cause: ICE::ArgumentTypeMismatchOnCall {
                    arg: *arg_kind,
                    expected: *obj_kind,
                },
                src: obj.symbols.source.source.clone(),
                err_span: obj.symbols.node_span(obj.ops[0].id).into(),
            }));
        }
    }
    // Allocate registers for the function call.
    let max_reg = obj.reg_max_index().0 + 1;
    let mut reg_stack = vec![None; max_reg + 1];
    // Copy the arguments into the appropriate registers
    for (ndx, arg) in arguments.into_iter().enumerate() {
        let r = obj.arguments[ndx];
        reg_stack[r.0] = Some(arg);
    }
    let mut state = VMState {
        reg_stack: &mut reg_stack,
        literals: &obj.literals,
        obj,
    };
    execute_block(&obj.ops, &mut state)?;
    match obj.return_slot {
        Slot::Empty => Ok(TypedBits::EMPTY),
        Slot::Register(r) => reg_stack
            .get(r.0)
            .cloned()
            .ok_or(rhdl_error(RHDLCompileError {
                cause: ICE::ReturnSlotNotFound {
                    name: format!("{:?}", r),
                },
                src: obj.symbols.source.source.clone(),
                err_span: obj.symbols.node_span(obj.ops[0].id).into(),
            }))?
            .ok_or(rhdl_error(RHDLCompileError {
                cause: ICE::ReturnSlotNotInitialized,
                src: obj.symbols.source.source.clone(),
                err_span: obj.symbols.node_span(obj.ops[0].id).into(),
            })),
        Slot::Literal(ndx) => Ok(obj.literals[&ndx].clone()),
    }
}
