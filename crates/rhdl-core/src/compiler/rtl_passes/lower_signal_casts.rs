use crate::{
    RHDLError,
    rtl::spec::AluUnary,
    rtl::{
        Object,
        object::LocatedOpCode,
        spec::{Assign, OpCode, Unary},
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerSignalCasts {}

fn lower_cast(lop: LocatedOpCode) -> LocatedOpCode {
    match lop.op {
        OpCode::Unary(Unary {
            op: AluUnary::Val,
            lhs,
            arg1,
        }) => LocatedOpCode {
            op: OpCode::Assign(Assign { lhs, rhs: arg1 }),
            loc: lop.loc,
        },
        _ => lop,
    }
}

impl Pass for LowerSignalCasts {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        input.ops = input.ops.into_iter().map(lower_cast).collect();
        Ok(input)
    }
    fn description() -> &'static str {
        "Lower signal casts to noop"
    }
}
