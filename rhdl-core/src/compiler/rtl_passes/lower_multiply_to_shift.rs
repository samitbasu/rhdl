use crate::{
    rhif::spec::AluBinary,
    rtl::{
        object::LocatedOpCode,
        spec::{Binary, Cast, CastKind, OpCode, Operand},
        Object,
    },
    types::bit_string::BitString,
    util::clog2,
    Digital, RHDLError,
};

use super::{allocate_literal, allocate_register, pass::Pass};

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
                        let literal = &input.literals[&lit];
                        let num_ones = literal.num_ones();
                        let trailing_zeros = literal.trailing_zeros();
                        if num_ones == 1 {
                            let literal_bits = clog2(trailing_zeros);
                            let literal_bs: BitString = (trailing_zeros as u8)
                                .typed_bits()
                                .unsigned_cast(literal_bits)
                                .unwrap()
                                .into();
                            let shift = allocate_literal(&mut input, lop.loc, literal_bs);
                            let lhs_kind = input.kind(binary.lhs);
                            let rhs_signed = input.kind(binary.arg1).is_signed();
                            // Allocate a register to hold the sign extended of the rhs
                            let r_extend = allocate_register(&mut input, lhs_kind, lop.loc);
                            input.ops.push(LocatedOpCode {
                                op: OpCode::Cast(Cast {
                                    lhs: Operand::Register(r_extend),
                                    arg: binary.arg1,
                                    len: lhs_kind.len(),
                                    kind: if rhs_signed {
                                        CastKind::Signed
                                    } else {
                                        CastKind::Unsigned
                                    },
                                }),
                                loc: lop.loc,
                            });
                            input.ops.push(LocatedOpCode {
                                op: OpCode::Binary(Binary {
                                    lhs: binary.lhs,
                                    op: AluBinary::Shl,
                                    arg1: Operand::Register(r_extend),
                                    arg2: Operand::Literal(shift),
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
}
