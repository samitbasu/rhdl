use crate::{
    compiler::mir::error::ICE,
    rtl::{
        object::{LocatedOpCode, RegisterKind},
        spec::{Concat, Index, OpCode, Operand},
        Object,
    },
    types::bit_string::BitString,
    RHDLError, TypedBits,
};

use super::{allocate_literal, allocate_register, pass::Pass};

#[derive(Default, Debug, Clone)]
pub struct LowerShiftByConstant {}

impl Pass for LowerShiftByConstant {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let ops = std::mem::take(&mut input.ops);
        for lop in ops {
            if let crate::rtl::spec::OpCode::Binary(crate::rtl::spec::Binary {
                op: crate::rhif::spec::AluBinary::Shl,
                lhs,
                arg1,
                arg2: crate::rtl::spec::Operand::Literal(lit),
            }) = lop.op
            {
                let shift_amount: TypedBits = (&input.literals[&lit]).into();
                let shift_amount = shift_amount.as_i64()?;
                if shift_amount < 0 {
                    return Err(Self::raise_ice(
                        &input,
                        ICE::ShiftOperatorRequiresUnsignedArgument,
                        lop.loc,
                    ));
                }
                let shift_amount = shift_amount as usize;
                let arg1_len = input.kind(arg1).len();
                let arg1_lsbs_len = arg1_len - shift_amount;
                // Allocate a new literal to hold the zeros shifted in on the right.
                let zero_lit =
                    allocate_literal(&mut input, lop.loc, BitString::zeros(shift_amount));
                let lsb_kind = if input.kind(arg1).is_signed() {
                    RegisterKind::Signed(arg1_lsbs_len)
                } else {
                    RegisterKind::Unsigned(arg1_lsbs_len)
                };
                let lsbs = allocate_register(&mut input, lsb_kind, lop.loc);
                input.ops.push(LocatedOpCode {
                    op: OpCode::Index(Index {
                        lhs: Operand::Register(lsbs),
                        arg: arg1,
                        bit_range: 0..arg1_lsbs_len,
                    }),
                    loc: lop.loc,
                });
                input.ops.push(LocatedOpCode {
                    op: OpCode::Concat(Concat {
                        lhs,
                        args: vec![Operand::Literal(zero_lit), Operand::Register(lsbs)],
                    }),
                    loc: lop.loc,
                });
            } else {
                input.ops.push(lop);
            }
        }
        Ok(input)
    }
}
