use crate::rhdl_core::{
    error::RHDLError,
    rhif::{
        spec::{Assign, LiteralId, OpCode, Slot},
        Object,
    },
    types::path::Path,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct PrecomputeDiscriminantPass {}

impl Pass for PrecomputeDiscriminantPass {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut new_literals = vec![];
        let literals = input.literals.clone();
        let mut max_literal = input.literal_max_index().0;
        let mut ops = input.ops.clone();
        for lop in ops.iter_mut() {
            if let OpCode::Index(index) = &lop.op {
                if index.path == Path::default().discriminant() {
                    if !input.kind(index.arg).is_enum() {
                        lop.op = OpCode::Assign(Assign {
                            lhs: index.lhs,
                            rhs: index.arg,
                        });
                    } else if let Slot::Literal(lit_id) = index.arg {
                        let literal_value = &literals[&lit_id];
                        let loc = input.symbols.slot_map[&index.arg];
                        let discriminant = literal_value.discriminant()?;
                        // Get a new literal slot for the discriminant
                        let discriminant_id = LiteralId(max_literal + 1);
                        let discriminant_slot = Slot::Literal(discriminant_id);
                        max_literal += 1;
                        new_literals.push((discriminant_id, discriminant, loc));
                        lop.op = OpCode::Assign(Assign {
                            lhs: index.lhs,
                            rhs: discriminant_slot,
                        });
                    }
                }
            }
        }
        for (id, value, loc) in new_literals {
            input.literals.insert(id, value);
            input.symbols.slot_map.insert(id.into(), loc);
        }
        input.ops = ops;
        Ok(input)
    }
}
