use std::collections::BTreeMap;

use crate::{
    compiler::mir::error::{RHDLCompileError, ICE},
    error::rhdl_error,
    rhif::{
        object::SourceLocation,
        spec::{AluBinary, AluUnary},
    },
    rtl::spec::{
        Case, CaseArgument, Cast, CastKind, Concat, DynamicIndex, DynamicSplice, Index, Select,
        Splice, Unary,
    },
    types::bit_string::BitString,
    RHDLError, TypedBits,
};

use super::{
    object::LocatedOpCode,
    spec::{Assign, Binary, LiteralId, OpCode, Operand},
    Object,
};

type Result<T> = core::result::Result<T, RHDLError>;

fn binary(op: AluBinary, arg1: BitString, arg2: BitString) -> Result<BitString> {
    let arg1: TypedBits = arg1.into();
    let arg2: TypedBits = arg2.into();
    let result = crate::rhif::runtime_ops::binary(op, arg1, arg2)?;
    Ok(result.into())
}

fn unary(op: AluUnary, arg1: BitString) -> Result<BitString> {
    let arg1: TypedBits = arg1.into();
    let result = crate::rhif::runtime_ops::unary(op, arg1)?;
    Ok(result.into())
}

struct VMState<'a> {
    reg_stack: &'a mut [Option<BitString>],
    literals: &'a BTreeMap<LiteralId, BitString>,
    obj: &'a Object,
}

impl<'a> VMState<'a> {
    fn raise_ice(&self, cause: ICE, loc: SourceLocation) -> RHDLError {
        let symbols = &self.obj.symbols[&loc.func];
        RHDLError::RHDLInternalCompilerError(Box::new(RHDLCompileError {
            cause,
            src: symbols.source.source.clone(),
            err_span: symbols.node_span(loc.node).into(),
        }))
    }
    fn read(&self, operand: Operand, loc: SourceLocation) -> Result<BitString> {
        match operand {
            Operand::Literal(l) => Ok(self.literals[&l].clone()),
            Operand::Register(r) => self.reg_stack[r.0]
                .clone()
                .ok_or(self.raise_ice(ICE::UninitializedRegister { r: r.0 }, loc)),
        }
    }
    fn write(&mut self, operand: Operand, value: BitString, loc: SourceLocation) -> Result<()> {
        match operand {
            Operand::Literal(ndx) => {
                Err(self.raise_ice(ICE::CannotWriteToLiteral { ndx: ndx.0 }, loc))
            }
            Operand::Register(r) => {
                self.reg_stack[r.0] = Some(value);
                Ok(())
            }
        }
    }
}

fn execute_block(ops: &[LocatedOpCode], state: &mut VMState) -> Result<()> {
    for lop in ops {
        let loc = lop.loc;
        let op = &lop.op;
        match op {
            OpCode::Assign(Assign { lhs, rhs }) => {
                let rhs = state.read(*rhs, loc)?;
                state.write(*lhs, rhs, loc)?;
            }
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
            OpCode::Case(Case {
                lhs,
                discriminant,
                table,
            }) => {
                let discriminant = state.read(*discriminant, loc)?;
                let arm = table
                    .iter()
                    .find(|(disc, _)| match disc {
                        CaseArgument::Literal(l) => discriminant == state.literals[l],
                        CaseArgument::Wild => true,
                    })
                    .ok_or(state.raise_ice(
                        ICE::NoMatchingArm {
                            discriminant: discriminant.into(),
                        },
                        loc,
                    ))?
                    .1;
                let arm = state.read(arm, loc)?;
                state.write(*lhs, arm, loc)?;
            }
            OpCode::Cast(Cast {
                lhs,
                arg,
                len,
                kind,
            }) => {
                let arg = state.read(*arg, loc)?;
                let result = match kind {
                    CastKind::Signed => arg.signed_cast(*len),
                    CastKind::Unsigned => arg.unsigned_cast(*len),
                }?;
                state.write(*lhs, result, loc)?;
            }
            OpCode::Comment(_) => {}
            OpCode::Concat(Concat { lhs, args }) => {
                let result = args
                    .iter()
                    .map(|a| state.read(*a, loc))
                    .collect::<Result<Vec<_>>>()?;
                let combined = result
                    .iter()
                    .flat_map(|x| x.bits())
                    .copied()
                    .collect::<Vec<bool>>();
                state.write(*lhs, BitString::Unsigned(combined), loc)?;
            }
            OpCode::DynamicIndex(DynamicIndex {
                lhs,
                arg,
                offset,
                len,
            }) => {
                let arg = state.read(*arg, loc)?;
                let offset = state.read(*offset, loc)?;
                let offset: TypedBits = offset.into();
                let offset = offset.as_i64()? as usize;
                let slice = arg.bits()[offset..(offset + *len)].to_vec();
                state.write(*lhs, BitString::Unsigned(slice), loc)?;
            }
            OpCode::DynamicSplice(DynamicSplice {
                lhs,
                arg,
                offset,
                len,
                value,
            }) => {
                let arg = state.read(*arg, loc)?;
                let value = state.read(*value, loc)?;
                let offset = state.read(*offset, loc)?;
                let offset: TypedBits = offset.into();
                let offset = offset.as_i64()? as usize;
                let mut arg = arg.bits().to_vec();
                let value = value.bits();
                arg.splice(offset..(offset + len), value.iter().copied());
                state.write(*lhs, BitString::Unsigned(arg), loc)?;
            }
            OpCode::Index(Index {
                lhs,
                arg,
                bit_range,
            }) => {
                let arg = state.read(*arg, loc)?;
                let slice = arg.bits()[bit_range.clone()].to_vec();
                state.write(*lhs, BitString::Unsigned(slice), loc)?;
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
                let result = if cond.bits().iter().any(|x| *x) {
                    true_value
                } else {
                    false_value
                };
                state.write(*lhs, result, loc)?;
            }
            OpCode::Splice(Splice {
                lhs,
                orig,
                value,
                bit_range,
            }) => {
                let orig = state.read(*orig, loc)?;
                let value = state.read(*value, loc)?;
                let mut orig = orig.bits().to_vec();
                let value = value.bits();
                orig.splice(bit_range.clone(), value.iter().copied());
                state.write(*lhs, BitString::Unsigned(orig), loc)?;
            }
            OpCode::Unary(Unary { op, lhs, arg1 }) => {
                let arg1 = state.read(*arg1, loc)?;
                let result = unary(*op, arg1)?;
                state.write(*lhs, result, loc)?;
            }
        }
    }
    Ok(())
}

pub fn execute(obj: &Object, arguments: Vec<BitString>) -> Result<BitString> {
    let symbols = &obj.symbols[&obj.fn_id];
    let loc = obj.ops[0].loc;
    // Load the object for this function
    if obj.arguments.len() != arguments.len() {
        return Err(rhdl_error(RHDLCompileError {
            cause: ICE::ArgumentCountMismatchOnCall,
            src: symbols.source.source.clone(),
            err_span: symbols.node_span(loc.node).into(),
        }));
    }
    for (ndx, arg) in arguments.iter().enumerate() {
        let arg_kind = &arg.kind;
        let obj_kind = &obj.kind[&obj.arguments[ndx]];
        if obj_kind != arg_kind {
            return Err(RHDLError::RHDLInternalCompilerError(Box::new(
                RHDLCompileError {
                    cause: ICE::ArgumentTypeMismatchOnCall {
                        arg: arg_kind.clone(),
                        expected: obj_kind.clone(),
                    },
                    src: obj.symbols.source.source.clone(),
                    err_span: obj.symbols.node_span(obj.ops[0].id).into(),
                },
            )));
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
            .ok_or(RHDLError::RHDLInternalCompilerError(Box::new(
                RHDLCompileError {
                    cause: ICE::ReturnSlotNotFound {
                        name: format!("{:?}", r),
                    },
                    src: obj.symbols.source.source.clone(),
                    err_span: obj.symbols.node_span(obj.ops[0].id).into(),
                },
            )))?
            .ok_or(RHDLError::RHDLInternalCompilerError(Box::new(
                RHDLCompileError {
                    cause: ICE::ReturnSlotNotInitialized,
                    src: obj.symbols.source.source.clone(),
                    err_span: obj.symbols.node_span(obj.ops[0].id).into(),
                },
            ))),
        Slot::Literal(ndx) => Ok(obj.literals[&ndx].clone()),
    }
}
