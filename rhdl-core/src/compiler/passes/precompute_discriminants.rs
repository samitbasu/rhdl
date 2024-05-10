use anyhow::Result;

use crate::{
    path::Path,
    rhif::{
        spec::{Assign, OpCode, Slot},
        Object,
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct PrecomputeDiscriminantPass {}

impl Pass for PrecomputeDiscriminantPass {
    fn name(&self) -> &'static str {
        "precompute_discriminants"
    }
    fn description(&self) -> &'static str {
        "Precompute discriminants"
    }
    fn run(mut input: Object) -> Result<Object> {
        let mut new_literals = vec![];
        let literals = input.literals.clone();
        let mut max_literal = input.literal_max_index();
        for op in input.ops.iter_mut() {
            if let OpCode::Index(index) = op {
                if index.path == Path::default().discriminant() {
                    if !input.kind[&index.arg].is_enum() {
                        *op = OpCode::Assign(Assign {
                            lhs: index.lhs,
                            rhs: index.arg,
                        });
                    } else if matches!(index.arg, Slot::Literal(_)) {
                        let literal_value = &literals[&index.arg];
                        let discriminant = literal_value.discriminant()?;
                        // Get a new literal slot for the discriminant
                        let discriminant_slot = Slot::Literal(max_literal + 1);
                        max_literal += 1;
                        new_literals.push((discriminant_slot, discriminant));
                        *op = OpCode::Assign(Assign {
                            lhs: index.lhs,
                            rhs: discriminant_slot,
                        });
                    }
                }
            }
        }
        for (slot, value) in new_literals {
            input.kind.insert(slot, value.kind.clone());
            input.literals.insert(slot, value);
        }
        Ok(input)
    }
}
