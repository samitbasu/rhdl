use crate::ast::SourceLocation;
use crate::ast::ast_impl::WrapOp;
use crate::common::slot_vec::SlotKey;
use crate::compiler::mir::error::{ICE, RHDLCompileError};
use crate::error::rhdl_error;
use crate::rhif::object::{Object, SourceDetails};
use crate::rhif::spec::{
    Array, Assign, Binary, Case, CaseArgument, Cast, Enum, Exec, Index, Member, OpCode, Repeat,
    Slot, Struct, Tuple, Unary,
};
use crate::types::path::Path;
use crate::{BitX, TypedBits};
use crate::{Kind, RHDLError};

use super::object::LocatedOpCode;
use super::runtime_ops::{array, binary, tuple, unary};
use super::spec::{Retime, Select, Splice, Wrap};

type Result<T> = std::result::Result<T, RHDLError>;

struct VMState<'a> {
    reg_stack: &'a mut [Option<TypedBits>],
    literals: &'a [(TypedBits, SourceDetails)],
    obj: &'a Object,
}

impl VMState<'_> {
    fn raise_ice(&self, cause: ICE, loc: SourceLocation) -> RHDLError {
        let symbols = &self.obj.symbols;
        rhdl_error(RHDLCompileError {
            cause,
            src: symbols.source(),
            err_span: symbols.span(loc).into(),
        })
    }
    fn read(&self, slot: Slot, loc: SourceLocation) -> Result<TypedBits> {
        match slot {
            Slot::Literal(l) => Ok(self.literals[l.index()].0.clone()),
            Slot::Register(r) => self.reg_stack[r.index()]
                .clone()
                .ok_or(self.raise_ice(ICE::UninitializedRegister { r }, loc)),
        }
    }
    fn write(&mut self, slot: Slot, value: TypedBits, loc: SourceLocation) -> Result<()> {
        match slot {
            Slot::Literal(ndx) => Err(self.raise_ice(ICE::CannotWriteToRHIFLiteral { ndx }, loc)),
            Slot::Register(r) => {
                self.reg_stack[r.index()] = Some(value);
                Ok(())
            }
        }
    }
    fn resolve_dynamic_paths(&mut self, path: &Path, loc: SourceLocation) -> Result<Path> {
        let mut result = Path::default();
        for element in path.iter() {
            match element {
                crate::types::path::PathElement::DynamicIndex(slot) => {
                    let slot = self.read(*slot, loc)?;
                    let ndx = slot.as_i64()?;
                    result = result.index(ndx as usize);
                }
                _ => result.push(element.clone()),
            }
        }
        Ok(result)
    }
}

fn execute_block(ops: &[LocatedOpCode], state: &mut VMState) -> Result<()> {
    for lop in ops {
        let op = &lop.op;
        let loc = lop.loc;
        match op {
            OpCode::Noop => {}
            OpCode::Binary(Binary {
                op,
                lhs,
                arg1,
                arg2,
            }) => {
                let arg1 = state.read(*arg1, loc)?;
                let arg2 = state.read(*arg2, loc)?;
                let result = binary(*op, arg1, arg2)?;
                state.write(*lhs, result, loc)?;
            }
            OpCode::Unary(Unary { op, lhs, arg1 }) => {
                let arg1 = state.read(*arg1, loc)?;
                let result = unary(*op, arg1)?;
                state.write(*lhs, result, loc)?;
            }
            OpCode::Select(Select {
                lhs,
                cond,
                true_value,
                false_value,
            }) => {
                let cond = state.read(*cond, loc)?;
                let true_value = state.read(*true_value, loc)?;
                let false_value = state.read(*false_value, loc)?;
                match cond.bits[0] {
                    BitX::Zero => state.write(*lhs, false_value, loc)?,
                    BitX::One => state.write(*lhs, true_value, loc)?,
                    BitX::X => state.write(*lhs, true_value.dont_care(), loc)?,
                }
            }
            OpCode::Index(Index { lhs, arg, path }) => {
                let arg = state.read(*arg, loc)?;
                let path = state.resolve_dynamic_paths(path, loc)?;
                let result = arg.path(&path)?;
                state.write(*lhs, result, loc)?;
            }
            OpCode::Splice(Splice {
                lhs,
                orig: rhs,
                path,
                subst: arg,
            }) => {
                let rhs_val = state.read(*rhs, loc)?;
                let path = state.resolve_dynamic_paths(path, loc)?;
                let arg_val = state.read(*arg, loc)?;
                let result = rhs_val.splice(&path, arg_val)?;
                state.write(*lhs, result, loc)?;
            }
            OpCode::Assign(Assign { lhs, rhs }) => {
                state.write(*lhs, state.read(*rhs, loc)?, loc)?;
            }
            OpCode::Tuple(Tuple { lhs, fields }) => {
                let fields = fields
                    .iter()
                    .map(|x| state.read(*x, loc))
                    .collect::<Result<Vec<_>>>()?;
                let result = tuple(&fields);
                state.write(*lhs, result, loc)?;
            }
            OpCode::Array(Array { lhs, elements }) => {
                let elements = elements
                    .iter()
                    .map(|x| state.read(*x, loc))
                    .collect::<Result<Vec<_>>>()?;
                let result = array(&elements);
                state.write(*lhs, result, loc)?;
            }
            OpCode::Struct(Struct {
                lhs,
                fields,
                rest,
                template,
            }) => {
                let mut result = if let Some(rest) = rest {
                    state.read(*rest, loc)?
                } else {
                    template.clone()
                };
                for field in fields {
                    let value = state.read(field.value, loc)?;
                    let path = match &field.member {
                        Member::Unnamed(ndx) => Path::default().tuple_index(*ndx as usize),
                        Member::Named(name) => Path::default().field(name),
                    };
                    result = result.splice(&path, value)?;
                }
                state.write(*lhs, result, loc)?;
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
                    let value = state.read(field.value, loc)?;
                    let path = match &field.member {
                        Member::Unnamed(ndx) => base_path.tuple_index(*ndx as usize),
                        Member::Named(name) => base_path.field(name),
                    };
                    result = result.splice(&path, value)?;
                }
                state.write(*lhs, result, loc)?;
            }
            OpCode::Case(Case {
                lhs,
                discriminant,
                table,
            }) => {
                let lhs_kind = state.obj.kind(*lhs);
                let lhs_dont_care = TypedBits::dont_care_from_kind(lhs_kind);
                let discriminant = state.read(*discriminant, loc)?;
                let arm = table
                    .iter()
                    .find(|(disc, _)| match disc {
                        CaseArgument::Slot(disc) => discriminant == state.read(*disc, loc).unwrap(),
                        CaseArgument::Wild => true,
                    })
                    .map(|x| x.1);
                let arm = if let Some(arm) = arm {
                    state.read(arm, loc)?
                } else {
                    lhs_dont_care
                };
                state.write(*lhs, arm, loc)?;
            }
            OpCode::AsBits(Cast { lhs, arg, len }) => {
                let arg = state.read(*arg, loc)?;
                let len = len.ok_or(state.raise_ice(ICE::BitCastMissingRequiredLength, loc))?;
                let result = arg.unsigned_cast(len)?;
                state.write(*lhs, result, loc)?;
            }
            OpCode::AsSigned(Cast { lhs, arg, len }) => {
                let arg = state.read(*arg, loc)?;
                let len = len.ok_or(state.raise_ice(ICE::BitCastMissingRequiredLength, loc))?;
                let result = arg.signed_cast(len)?;
                state.write(*lhs, result, loc)?;
            }
            OpCode::Resize(Cast { lhs, arg, len }) => {
                let arg = state.read(*arg, loc)?;
                let len = len.ok_or(state.raise_ice(ICE::BitCastMissingRequiredLength, loc))?;
                let result = arg.resize(len)?;
                state.write(*lhs, result, loc)?;
            }
            OpCode::Retime(Retime { lhs, arg, color }) => {
                let mut arg = state.read(*arg, loc)?;
                if let Some(color) = color {
                    arg.kind = Kind::make_signal(arg.kind, *color);
                }
                state.write(*lhs, arg, loc)?;
            }
            OpCode::Wrap(Wrap { op, lhs, arg, kind }) => {
                let arg = state.read(*arg, loc)?;
                let Some(kind) = kind else {
                    return Err(state.raise_ice(ICE::WrapMissingKind, loc));
                };
                let arg = match op {
                    WrapOp::Ok => arg.wrap_ok(kind),
                    WrapOp::Err => arg.wrap_err(kind),
                    WrapOp::Some => arg.wrap_some(kind),
                    WrapOp::None => arg.wrap_none(kind),
                }?;
                state.write(*lhs, arg, loc)?;
            }
            OpCode::Exec(Exec {
                lhs,
                id: f_id,
                args,
            }) => {
                let args = args
                    .iter()
                    .map(|x| state.read(*x, loc))
                    .collect::<Result<Vec<_>>>()?;
                let func = &state.obj.externals[f_id];
                let result = execute(func, args)?;
                state.write(*lhs, result, loc)?;
            }
            OpCode::Repeat(Repeat { lhs, value, len }) => {
                let value = state.read(*value, loc)?;
                let len = *len as usize;
                let result = value.repeat(len);
                state.write(*lhs, result, loc)?;
            }
        }
    }
    Ok(())
}

pub fn execute(obj: &Object, arguments: Vec<TypedBits>) -> Result<TypedBits> {
    let symbols = &obj.symbols;
    let loc = symbols.fallback(obj.fn_id);
    // Load the object for this function
    if obj.arguments.len() != arguments.len() {
        return Err(rhdl_error(RHDLCompileError {
            cause: ICE::ArgumentCountMismatchOnCall,
            src: symbols.source(),
            err_span: symbols.span(loc).into(),
        }));
    }
    for (ndx, arg) in arguments.iter().enumerate() {
        let arg_kind = &arg.kind;
        let obj_kind = &obj.symtab[obj.arguments[ndx]];
        if obj_kind != arg_kind {
            return Err(rhdl_error(RHDLCompileError {
                cause: ICE::ArgumentTypeMismatchOnCall {
                    arg: *arg_kind,
                    expected: *obj_kind,
                },
                src: symbols.source(),
                err_span: symbols.span(loc).into(),
            }));
        }
    }
    // Allocate registers for the function call.
    let max_reg = obj.symtab.reg_vec().len();
    let mut reg_stack = vec![None; max_reg];
    // Copy the arguments into the appropriate registers
    for (ndx, arg) in arguments.into_iter().enumerate() {
        let r = obj.arguments[ndx];
        reg_stack[r.index()] = Some(arg);
    }
    let mut state = VMState {
        reg_stack: &mut reg_stack,
        literals: obj.symtab.lit_vec(),
        obj,
    };
    execute_block(&obj.ops, &mut state)?;
    match obj.return_slot {
        Slot::Register(r) => reg_stack
            .get(r.index())
            .cloned()
            .ok_or(rhdl_error(RHDLCompileError {
                cause: ICE::ReturnSlotNotFound {
                    name: format!("{r:?}"),
                },
                src: symbols.source(),
                err_span: symbols.span(loc).into(),
            }))?
            .ok_or(rhdl_error(RHDLCompileError {
                cause: ICE::ReturnSlotNotInitialized,
                src: symbols.source(),
                err_span: symbols.span(loc).into(),
            })),
        Slot::Literal(ndx) => Ok(obj.symtab[ndx].clone()),
    }
}
