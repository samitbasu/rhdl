use crate::rhdl_core::{
    rtl::{
        object::LocatedOpCode,
        spec::{Assign, OpCode, Operand},
        Object,
    },
    types::bit_string::BitString,
    RHDLError,
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
            .filter(|x| input.register_kind[*x].is_empty())
            .copied()
            .collect::<Vec<_>>();
        if empty_args.is_empty() {
            return Ok(input);
        }
        let my_empty = input.literal_max_index().next();
        input.literals.insert(my_empty, BitString::Unsigned(vec![]));
        let my_empty = Operand::Literal(my_empty);
        let fallback = input.symbols.fallback(input.fn_id);
        input.symbols.operand_map.insert(my_empty, fallback);
        let preamble = empty_args
            .iter()
            .map(|arg| LocatedOpCode {
                op: OpCode::Assign(Assign {
                    lhs: Operand::Register(*arg),
                    rhs: my_empty,
                }),
                loc: fallback,
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
}
