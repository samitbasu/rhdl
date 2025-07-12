use crate::rhdl_core::{
    RHDLError, TypedBits,
    rtl::{
        Object,
        object::LocatedOpCode,
        spec::{Assign, OpCode, Operand},
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveEmptyFunctionArguments {}

impl Pass for RemoveEmptyFunctionArguments {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let empty_args = input
            .arguments
            .iter()
            .flat_map(|x| x.as_ref())
            .filter(|x| input.symtab[*x].is_empty())
            .copied()
            .collect::<Vec<_>>();
        if empty_args.is_empty() {
            return Ok(input);
        }
        let preamble = empty_args
            .iter()
            .map(|arg| {
                let operand = Operand::Register(*arg);
                let details = input.symtab[operand].clone();
                let loc = details.location;
                let empty = input.symtab.lit(TypedBits::EMPTY, details);
                LocatedOpCode {
                    op: OpCode::Assign(Assign {
                        lhs: Operand::Register(*arg),
                        rhs: empty,
                    }),
                    loc,
                }
            })
            .collect::<Vec<_>>();
        input.ops = preamble.into_iter().chain(input.ops).collect();
        input.arguments = input
            .arguments
            .into_iter()
            .map(|x| x.filter(|&x| !empty_args.contains(&x)))
            .collect();
        Ok(input)
    }
    fn description() -> &'static str {
        "Remove empty function arguments"
    }
}
