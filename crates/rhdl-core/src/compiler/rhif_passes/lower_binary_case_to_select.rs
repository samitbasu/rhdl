use crate::{
    RHDLError,
    common::symtab::Symbol,
    rhif::{
        Object,
        spec::{CaseArgument, OpCode, Select},
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerBinaryCaseToSelectPass {}

impl Pass for LowerBinaryCaseToSelectPass {
    fn description() -> &'static str {
        "Lower binary case statements to select statements"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut ops = std::mem::take(&mut input.ops);
        for lop in ops.iter_mut() {
            if let OpCode::Case(case) = &lop.op
                // Discriminant is a single bit
                && input.kind(case.discriminant).bits() == 1
                // There are precisely two cases
                && case.table.len() == 2
                // Both cases are literals
                && let CaseArgument::Slot(Symbol::Literal(lid0)) = case.table[0].0
                && let CaseArgument::Slot(Symbol::Literal(lid1)) = case.table[1].0
                // The two literals are different
                && let Ok(l0) = input.symtab[lid0].as_bool()
                && let Ok(l1) = input.symtab[lid1].as_bool()
                && (l0 ^ l1)
            {
                let true_value = if l0 { case.table[0].1 } else { case.table[1].1 };
                let false_value = if l0 { case.table[1].1 } else { case.table[0].1 };
                let select = OpCode::Select(Select {
                    lhs: case.lhs,
                    cond: case.discriminant,
                    true_value,
                    false_value,
                });
                lop.op = select;
            }
        }
        input.ops = ops;
        Ok(input)
    }
}
