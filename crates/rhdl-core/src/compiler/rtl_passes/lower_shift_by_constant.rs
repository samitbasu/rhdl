use crate::{
    Kind, RHDLError, TypedBits,
    ast::source::source_location::SourceLocation,
    common::symtab::LiteralId,
    compiler::mir::error::ICE,
    rtl::{
        Object,
        object::LocatedOpCode,
        spec::{AluBinary, Binary, Cast, CastKind, Concat, Index, OpCode, Operand, OperandKind},
    },
    types::{bit_string::BitString, path::Path},
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerShiftByConstant {}

impl LowerShiftByConstant {
    fn shift_amount_as_usize(
        input: &Object,
        lit: LiteralId<OperandKind>,
        loc: SourceLocation,
    ) -> Result<usize, RHDLError> {
        let shift_amount: TypedBits = input.symtab[&lit].clone();
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
        lit: LiteralId<OperandKind>,
        loc: SourceLocation,
    ) -> Result<(), RHDLError> {
        let shift_amount = Self::shift_amount_as_usize(input, lit, loc)?;
        let source_details = input.symtab[arg1].clone();
        let arg1_len = input.kind(arg1).bits();
        let arg1_ext_len = arg1_len + shift_amount;
        let ext_kind = if input.kind(lhs).is_signed() {
            Kind::Signed(arg1_ext_len)
        } else {
            Kind::Bits(arg1_ext_len)
        };
        let ext = input.symtab.reg(ext_kind, source_details);
        input.ops.push(LocatedOpCode {
            op: OpCode::Cast(Cast {
                lhs: ext,
                arg: arg1,
                kind: CastKind::Resize,
                len: arg1_ext_len,
            }),
            loc,
        });
        input.ops.push(LocatedOpCode {
            op: OpCode::Index(Index {
                lhs,
                arg: ext,
                bit_range: shift_amount..arg1_ext_len,
                path: Path::default(),
            }),
            loc,
        });
        Ok(())
    }

    fn lower_left_shift(
        input: &mut Object,
        lhs: Operand,
        arg1: Operand,
        lit: LiteralId<OperandKind>,
        loc: SourceLocation,
    ) -> Result<(), RHDLError> {
        let shift_amount = Self::shift_amount_as_usize(input, lit, loc)?;
        let arg1_len = input.kind(arg1).bits();
        let arg1_lsbs_len = arg1_len.saturating_sub(shift_amount);
        // Allocate a new literal to hold the zeros shifted in on the right.
        let source_details = input.symtab[arg1].clone();
        let zero_kind = Kind::Bits(shift_amount);
        let zero_bits = BitString::zeros(shift_amount);
        let zero_tb = TypedBits {
            kind: zero_kind,
            bits: zero_bits.bits().to_vec(),
        };
        let zero_lit = input.symtab.lit(zero_tb, source_details.clone());
        let lsb_kind = if input.kind(arg1).is_signed() {
            Kind::Signed(arg1_lsbs_len)
        } else {
            Kind::Bits(arg1_lsbs_len)
        };
        let lsbs = input.symtab.reg(lsb_kind, source_details);
        input.ops.push(LocatedOpCode {
            op: OpCode::Index(Index {
                lhs: lsbs,
                arg: arg1,
                bit_range: 0..arg1_lsbs_len,
                path: Path::default(),
            }),
            loc,
        });
        input.ops.push(LocatedOpCode {
            op: OpCode::Concat(Concat {
                lhs,
                args: vec![zero_lit, lsbs],
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
    fn description() -> &'static str {
        "Lower shift by a constant to index/concat operation"
    }
}
