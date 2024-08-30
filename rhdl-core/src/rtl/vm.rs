use std::collections::BTreeMap;

use crate::{
    ast::ast_impl::NodeId,
    compiler::mir::error::{RHDLCompileError, ICE},
    rhif::object::SourceLocation,
    RHDLError,
};

use super::{
    object::{BitString, LocatedOpCode},
    spec::{Assign, Binary, LiteralId, OpCode, Operand},
    Object,
};

type Result<T> = core::result::Result<T, RHDLError>;

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
        /* match op {
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
        }*/
    }
    todo!()
}
