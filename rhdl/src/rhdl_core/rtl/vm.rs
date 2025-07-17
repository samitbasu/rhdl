use crate::rhdl_core::{
    RHDLError, TypedBits,
    ast::source::source_location::SourceLocation,
    bitx::BitX,
    common::slot_vec::SlotKey,
    compiler::mir::error::{ICE, RHDLCompileError},
    error::rhdl_error,
    rhif::object::SourceDetails,
    rtl::spec::{
        AluBinary, AluUnary, Case, CaseArgument, Cast, CastKind, Concat, Index, Select, Splice,
        Unary,
    },
    types::bit_string::BitString,
};

use super::{
    Object,
    object::LocatedOpCode,
    runtime_ops::{binary, unary},
    spec::{Assign, Binary, OpCode, Operand},
};

type Result<T> = core::result::Result<T, RHDLError>;

struct VMState<'a> {
    reg_stack: &'a mut [Option<BitString>],
    literals: &'a [(TypedBits, SourceDetails)],
    obj: &'a Object,
}

impl VMState<'_> {
    fn raise_ice(&self, cause: ICE, loc: SourceLocation) -> RHDLError {
        let symbols = &self.obj.symbols;
        RHDLError::RHDLInternalCompilerError(Box::new(RHDLCompileError {
            cause,
            src: symbols.source(),
            err_span: symbols.span(loc).into(),
        }))
    }
    fn binary(
        &self,
        op: AluBinary,
        arg1: BitString,
        arg2: BitString,
        loc: SourceLocation,
    ) -> Result<BitString> {
        let arg1: TypedBits = arg1.into();
        let arg2: TypedBits = arg2.into();
        match binary(op, arg1, arg2) {
            Ok(result) => Ok(result.into()),
            Err(e) => Err(self.raise_ice(ICE::BinaryOperatorError(Box::new(e)), loc)),
        }
    }

    fn unary(&self, op: AluUnary, arg1: BitString, loc: SourceLocation) -> Result<BitString> {
        let arg1: TypedBits = arg1.into();
        match unary(op, arg1) {
            Ok(result) => Ok(result.into()),
            Err(e) => Err(self.raise_ice(ICE::UnaryOperatorError(Box::new(e)), loc)),
        }
    }
    fn read(&self, operand: Operand, loc: SourceLocation) -> Result<BitString> {
        match operand {
            Operand::Literal(l) => Ok((&self.literals[l.index()].0).into()),
            Operand::Register(r) => self.reg_stack[r.index()]
                .clone()
                .ok_or(self.raise_ice(ICE::UninitializedRTLRegister { r }, loc)),
        }
    }
    fn write(&mut self, operand: Operand, value: BitString, loc: SourceLocation) -> Result<()> {
        match operand {
            Operand::Literal(ndx) => Err(self.raise_ice(ICE::CannotWriteToRTLLiteral { ndx }, loc)),
            Operand::Register(r) => {
                self.reg_stack[r.index()] = Some(value);
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
            OpCode::Noop => {}
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
                let result = state.binary(*op, arg1, arg2, loc)?;
                state.write(*lhs, result, loc)?;
            }
            OpCode::Case(Case {
                lhs,
                discriminant,
                table,
            }) => {
                let lhs_kind = state.obj.kind(*lhs);
                let lhs_dont_care = BitString::dont_care_from_kind(lhs_kind);
                let discriminant = state.read(*discriminant, loc)?;
                let arm = table
                    .iter()
                    .find(|(disc, _)| match disc {
                        CaseArgument::Literal(l) => {
                            discriminant == (&state.literals[l.index()].0).into()
                        }
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
                    CastKind::Resize => arg.resize(*len),
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
                    .collect::<Vec<BitX>>();
                if state.obj.kind(*lhs).is_signed() {
                    state.write(*lhs, BitString::Signed(combined), loc)?;
                } else {
                    state.write(*lhs, BitString::Unsigned(combined), loc)?;
                }
            }
            OpCode::Index(Index {
                lhs,
                arg,
                bit_range,
                path: _,
            }) => {
                let arg = state.read(*arg, loc)?;
                let slice = arg.bits()[bit_range.clone()].to_vec();
                if state.obj.kind(*lhs).is_signed() {
                    state.write(*lhs, BitString::Signed(slice), loc)?;
                } else {
                    state.write(*lhs, BitString::Unsigned(slice), loc)?;
                }
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
                match cond.bits()[0] {
                    BitX::Zero => state.write(*lhs, false_value, loc)?,
                    BitX::One => state.write(*lhs, true_value, loc)?,
                    BitX::X => state.write(*lhs, true_value.dont_care(), loc)?,
                }
            }
            OpCode::Splice(Splice {
                lhs,
                orig,
                value,
                bit_range,
                path: _,
            }) => {
                let orig = state.read(*orig, loc)?;
                let value = state.read(*value, loc)?;
                let mut orig = orig.bits().to_vec();
                let value = value.bits();
                orig.splice(bit_range.clone(), value.iter().copied());
                if state.obj.kind(*lhs).is_signed() {
                    state.write(*lhs, BitString::Signed(orig), loc)?;
                } else {
                    state.write(*lhs, BitString::Unsigned(orig), loc)?;
                }
            }
            OpCode::Unary(Unary { op, lhs, arg1 }) => {
                let arg1 = state.read(*arg1, loc)?;
                let result = state.unary(*op, arg1, loc)?;
                state.write(*lhs, result, loc)?;
            }
        }
    }
    Ok(())
}

pub fn execute(obj: &Object, arguments: Vec<BitString>) -> Result<BitString> {
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
        if obj.arguments[ndx].is_none() ^ arg.is_empty() {
            return Err(rhdl_error(RHDLCompileError {
                cause: ICE::NonemptyToEmptyArgumentMismatch,
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
        if let Some(r) = obj.arguments[ndx] {
            reg_stack[r.index()] = Some(arg);
        }
    }
    let mut state = VMState {
        reg_stack: &mut reg_stack,
        literals: obj.symtab.lit_vec(),
        obj,
    };
    execute_block(&obj.ops, &mut state)?;
    match obj.return_register {
        Operand::Register(r) => reg_stack
            .get(r.index())
            .cloned()
            .ok_or(RHDLError::RHDLInternalCompilerError(Box::new(
                RHDLCompileError {
                    cause: ICE::ReturnSlotNotFound {
                        name: format!("{r:?}"),
                    },
                    src: symbols.source(),
                    err_span: symbols.span(loc).into(),
                },
            )))?
            .ok_or(RHDLError::RHDLInternalCompilerError(Box::new(
                RHDLCompileError {
                    cause: ICE::ReturnSlotNotInitialized,
                    src: symbols.source(),
                    err_span: symbols.span(loc).into(),
                },
            ))),
        Operand::Literal(ndx) => Ok((&obj.symtab[ndx]).into()),
    }
}
