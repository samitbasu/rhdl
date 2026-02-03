use crate::{
    RHDLError,
    rtl::spec::AluBinary,
    rtl::{
        Object,
        object::LocatedOpCode,
        spec::{AluUnary, Binary, OpCode, Operand, Unary},
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerNotEqualZeroToAny {}

fn replace_not_equal_zero(input: &Object, lop: LocatedOpCode) -> LocatedOpCode {
    if let OpCode::Binary(Binary {
        op: AluBinary::Ne,
        lhs,
        arg1,
        arg2,
    }) = lop.op
    {
        if let Operand::Literal(l1) = arg1 {
            if input.symtab[&l1].is_zero() {
                return LocatedOpCode {
                    op: OpCode::Unary(Unary {
                        lhs,
                        op: AluUnary::Any,
                        arg1: arg2,
                    }),
                    loc: lop.loc,
                };
            }
        } else if let Operand::Literal(l2) = arg2
            && input.symtab[&l2].is_zero()
        {
            return LocatedOpCode {
                op: OpCode::Unary(Unary {
                    lhs,
                    op: AluUnary::Any,
                    arg1,
                }),
                loc: lop.loc,
            };
        }
    }
    lop
}

impl Pass for LowerNotEqualZeroToAny {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let ops = std::mem::take(&mut input.ops);
        input.ops = ops
            .into_iter()
            .map(|lop| replace_not_equal_zero(&input, lop))
            .collect();
        Ok(input)
    }
    fn description() -> &'static str {
        "Lower NotEqual to Any"
    }
}
