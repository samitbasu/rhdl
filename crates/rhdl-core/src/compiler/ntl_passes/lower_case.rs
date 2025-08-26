use super::pass::Pass;

use crate::core::ntl::object::*;
use crate::core::ntl::spec::*;
use crate::prelude::RHDLError;

pub struct LowerCase;

fn lower_case(op: &mut OpCode) {
    let OpCode::Case(case) = op else { return };
    if case.entries.len() != 2 {
        return;
    }
    if case.discriminant.len() != 1 {
        return;
    }
    let selector = case.discriminant[0];
    let arg0 = &case.entries[0];
    let arg1 = &case.entries[1];
    match (&arg0.0, &arg1.0) {
        (CaseEntry::Literal(l0), CaseEntry::Literal(l1)) => {
            if l0.is_ones() && l1.is_zero() {
                *op = OpCode::Select(Select {
                    lhs: case.lhs,
                    selector,
                    true_case: arg0.1,
                    false_case: arg1.1,
                })
            } else if l0.is_zero() && l1.is_ones() {
                *op = OpCode::Select(Select {
                    lhs: case.lhs,
                    selector,
                    true_case: arg1.1,
                    false_case: arg0.1,
                })
            }
        }
        (CaseEntry::Literal(l0), CaseEntry::WildCard) => {
            if l0.is_ones() {
                *op = OpCode::Select(Select {
                    lhs: case.lhs,
                    selector,
                    true_case: arg0.1,
                    false_case: arg1.1,
                })
            } else if l0.is_zero() {
                *op = OpCode::Select(Select {
                    lhs: case.lhs,
                    selector,
                    true_case: arg1.1,
                    false_case: arg0.1,
                })
            }
        }
        _ => {}
    }
}

impl Pass for LowerCase {
    fn description() -> &'static str {
        "Lower Case opcodes where possible"
    }

    fn run(mut input: Object) -> Result<Object, RHDLError> {
        input.ops.iter_mut().for_each(|lop| {
            lower_case(&mut lop.op);
        });
        Ok(input)
    }
}
