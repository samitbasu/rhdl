use std::collections::HashMap;

use crate::{
    RHDLError,
    rhif::{
        Object,
        spec::{Assign, OpCode, Slot},
        visit::visit_object_slots_mut,
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct PropagateLiteralsPass {}

impl Pass for PropagateLiteralsPass {
    fn description() -> &'static str {
        "Propagate literals through registers"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        // A candidate is an assignment of the form rx <- ly
        let candidates = input
            .ops
            .iter()
            .filter_map(|lop| {
                if let OpCode::Assign(Assign {
                    lhs: Slot::Register(rid),
                    rhs: Slot::Literal(lid),
                }) = &lop.op
                {
                    Some((*rid, *lid))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>();
        visit_object_slots_mut(&mut input, |sense, slot| {
            if sense.is_read()
                && let Some(rid) = slot.reg()
                && let Some(lid) = candidates.get(&rid)
            {
                *slot = Slot::Literal(*lid);
            }
        });
        input.ops.retain(|lop| {
            if let OpCode::Assign(Assign { lhs, rhs }) = lop.op {
                lhs != rhs
            } else {
                true
            }
        });
        Ok(input)
    }
}
