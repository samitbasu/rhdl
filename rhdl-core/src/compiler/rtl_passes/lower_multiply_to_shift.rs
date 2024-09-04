use crate::{
    rhif::spec::AluBinary,
    rtl::{
        spec::{OpCode, Operand},
        Object,
    },
    RHDLError,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerMultiplyToShift {}

impl Pass for LowerMultiplyToShift {
    fn name() -> &'static str {
        "lower_multiply_to_shift"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut ops = std::mem::take(&mut input.ops);
        for lop in ops.iter_mut() {
            if let OpCode::Binary(binary) = &mut lop.op {
                if binary.op == AluBinary::Mul {
                    if let Operand::Literal(lit) = binary.arg2 {
                        let literal = &input.literals[&lit];
                        let num_ones = literal.num_ones();
                        let trailing_zeros = literal.trailing_zeros();
                    }
                }
            }
        }
        input.ops = ops;
        Ok(input)
    }
}
