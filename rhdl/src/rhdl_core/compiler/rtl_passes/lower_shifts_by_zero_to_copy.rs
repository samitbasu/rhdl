use crate::rhdl_core::{
    rtl::spec::AluBinary,
    rtl::{
        object::LocatedOpCode,
        spec::{Assign, Binary, OpCode, Operand},
        Object,
    },
    RHDLError,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerShiftsByZeroToCopy {}

fn replace_shift_by_zero(input: &Object, lop: LocatedOpCode) -> LocatedOpCode {
    if let OpCode::Binary(Binary {
        op: AluBinary::Shl | AluBinary::Shr,
        lhs,
        arg1,
        arg2: Operand::Literal(lit),
    }) = lop.op
    {
        if input.literals[&lit].is_zero() {
            return LocatedOpCode {
                op: OpCode::Assign(Assign { lhs, rhs: arg1 }),
                loc: lop.loc,
            };
        }
    }
    lop
}

impl Pass for LowerShiftsByZeroToCopy {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let ops = std::mem::take(&mut input.ops);
        input.ops = ops
            .into_iter()
            .map(|lop| replace_shift_by_zero(&input, lop))
            .collect();
        Ok(input)
    }
}
