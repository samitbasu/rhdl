use crate::{
    error::RHDLError,
    rhif::{
        Object,
        spec::{Assign, OpCode, Slot},
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveUnneededMuxesPass {}

impl Pass for RemoveUnneededMuxesPass {
    fn description() -> &'static str {
        "Remove unneeded muxes (literal selector or equal branches)"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut ops = std::mem::take(&mut input.ops);
        for lop in ops.iter_mut() {
            if let OpCode::Select(select) = lop.op.clone() {
                if let Some(literal) = select.cond.lit() {
                    let val = &input.symtab[literal];
                    if val.as_bool()? {
                        lop.op = OpCode::Assign(Assign {
                            lhs: select.lhs,
                            rhs: select.true_value,
                        });
                    } else {
                        lop.op = OpCode::Assign(Assign {
                            lhs: select.lhs,
                            rhs: select.false_value,
                        });
                    }
                } else if select.true_value == select.false_value {
                    lop.op = OpCode::Assign(Assign {
                        lhs: select.lhs,
                        rhs: select.true_value,
                    });
                } else if let Slot::Literal(true_lit) = select.true_value
                    && let Slot::Literal(false_lit) = select.false_value
                    && input.symtab[true_lit] == input.symtab[false_lit]
                {
                    lop.op = OpCode::Assign(Assign {
                        lhs: select.lhs,
                        rhs: select.true_value,
                    });
                }
            }
        }
        input.ops = ops;
        Ok(input)
    }
}
