use core::panic;

use crate::{
    ast::source_location::SourceLocation,
    compiler::mir::error::ICE,
    rhif::spec::AluBinary,
    rtl::{
        object::{LocatedOpCode, RegisterKind},
        spec::{Binary, Cast, CastKind, Concat, Index, LiteralId, OpCode, Operand},
        Object,
    },
    types::bit_string::BitString,
    RHDLError, TypedBits,
};

use super::{allocate_literal, allocate_register, pass::Pass};

#[derive(Default, Debug, Clone)]
pub struct LowerShiftByConstant {}

impl LowerShiftByConstant {
    fn shift_amount_as_usize(
        input: &Object,
        lit: LiteralId,
        loc: SourceLocation,
    ) -> Result<usize, RHDLError> {
        let shift_amount: TypedBits = (&input.literals[&lit]).into();
        let shift_amount = shift_amount.as_i64()?;
        if shift_amount < 0 {
            return Err(Self::raise_ice(
                input,
                ICE::ShiftOperatorRequiresUnsignedArgumentConstant {
                    shift: shift_amount,
                },
                loc,
            ));
        }
        Ok(shift_amount as usize)
    }

    fn lower_right_shift(
        input: &mut Object,
        lhs: Operand,
        arg1: Operand,
        lit: LiteralId,
        loc: SourceLocation,
    ) -> Result<(), RHDLError> {
        let shift_amount = Self::shift_amount_as_usize(input, lit, loc)?;
        let arg1_len = input.kind(arg1).len();
        let arg1_ext_len = arg1_len + shift_amount;
        let ext_kind = if input.kind(arg1).is_signed() {
            RegisterKind::Signed(arg1_ext_len)
        } else {
            RegisterKind::Unsigned(arg1_ext_len)
        };
        let ext = allocate_register(input, ext_kind, loc);
        input.ops.push(LocatedOpCode {
            op: OpCode::Cast(Cast {
                lhs: Operand::Register(ext),
                arg: arg1,
                kind: CastKind::Resize,
                len: arg1_ext_len,
            }),
            loc,
        });
        input.ops.push(LocatedOpCode {
            op: OpCode::Index(Index {
                lhs,
                arg: Operand::Register(ext),
                bit_range: shift_amount..arg1_ext_len,
            }),
            loc,
        });
        Ok(())
    }

    fn lower_left_shift(
        input: &mut Object,
        lhs: Operand,
        arg1: Operand,
        lit: LiteralId,
        loc: SourceLocation,
    ) -> Result<(), RHDLError> {
        let shift_amount = Self::shift_amount_as_usize(input, lit, loc)?;
        let arg1_len = input.kind(arg1).len();
        let arg1_lsbs_len = arg1_len.saturating_sub(shift_amount);
        // Allocate a new literal to hold the zeros shifted in on the right.
        let zero_lit = allocate_literal(input, loc, BitString::zeros(shift_amount));
        let lsb_kind = if input.kind(arg1).is_signed() {
            RegisterKind::Signed(arg1_lsbs_len)
        } else {
            RegisterKind::Unsigned(arg1_lsbs_len)
        };
        let lsbs = allocate_register(input, lsb_kind, loc);
        input.ops.push(LocatedOpCode {
            op: OpCode::Index(Index {
                lhs: Operand::Register(lsbs),
                arg: arg1,
                bit_range: 0..arg1_lsbs_len,
            }),
            loc,
        });
        input.ops.push(LocatedOpCode {
            op: OpCode::Concat(Concat {
                lhs,
                args: vec![Operand::Literal(zero_lit), Operand::Register(lsbs)],
            }),
            loc,
        });
        Ok(())
    }
}

impl Pass for LowerShiftByConstant {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let ops = std::mem::take(&mut input.ops);
        for lop in ops {
            match lop.op {
                OpCode::Binary(Binary {
                    op: AluBinary::Shl,
                    lhs,
                    arg1,
                    arg2: Operand::Literal(lit),
                }) => {
                    Self::lower_left_shift(&mut input, lhs, arg1, lit, lop.loc)?;
                }
                OpCode::Binary(Binary {
                    op: AluBinary::Shr,
                    lhs,
                    arg1,
                    arg2: Operand::Literal(lit),
                }) => {
                    Self::lower_right_shift(&mut input, lhs, arg1, lit, lop.loc)?;
                }
                _ => {
                    input.ops.push(lop);
                }
            }
        }
        Ok(input)
    }
}
