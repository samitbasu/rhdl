use crate::rhdl_bits::alias::b8;

use crate::rhdl_core::TypedBits;
use crate::rhdl_core::{
    Digital, RHDLError,
    rtl::spec::AluBinary,
    rtl::{
        Object,
        object::LocatedOpCode,
        spec::{Binary, Cast, CastKind, OpCode, Operand},
    },
    util::clog2,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerMultiplyToShift {}

impl Pass for LowerMultiplyToShift {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let ops = std::mem::take(&mut input.ops);
        for lop in ops {
            let mut replaced = false;
            if let OpCode::Binary(binary) = &lop.op {
                if binary.op == AluBinary::Mul {
                    if let Operand::Literal(lit) = binary.arg2 {
                        let literal = &input.symtab[&lit];
                        let num_ones = literal.num_ones();
                        let trailing_zeros = literal.trailing_zeros();
                        if num_ones == 1 {
                            let literal_bits = clog2(trailing_zeros);
                            let literal_bs: TypedBits = b8(trailing_zeros as u128)
                                .typed_bits()
                                .unsigned_cast(literal_bits)
                                .unwrap();
                            let source_details = input.symtab[&Operand::Literal(lit)].clone();
                            let shift = input.symtab.lit(literal_bs, source_details.clone());
                            let lhs_kind = input.kind(binary.lhs);
                            // Allocate a register to hold the sign extended of the rhs
                            let r_extend = input.symtab.reg(lhs_kind, source_details);
                            input.ops.push(LocatedOpCode {
                                op: OpCode::Cast(Cast {
                                    lhs: r_extend,
                                    arg: binary.arg1,
                                    len: lhs_kind.bits(),
                                    kind: CastKind::Resize,
                                }),
                                loc: lop.loc,
                            });
                            input.ops.push(LocatedOpCode {
                                op: OpCode::Binary(Binary {
                                    lhs: binary.lhs,
                                    op: AluBinary::Shl,
                                    arg1: r_extend,
                                    arg2: shift,
                                }),
                                loc: lop.loc,
                            });
                            replaced = true;
                        }
                    }
                }
            }
            if !replaced {
                input.ops.push(lop);
            }
        }
        Ok(input)
    }
    fn description() -> &'static str {
        "Lower multiply by 2^N to a shift"
    }
}
