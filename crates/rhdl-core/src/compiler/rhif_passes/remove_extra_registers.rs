use std::collections::HashMap;

use super::pass::Pass;
use crate::{
    common::{symtab::RegisterId, unify_key::EnaKey},
    error::RHDLError,
    rhif::{
        Object,
        spec::{Assign, OpCode, Slot},
        visit::visit_object_slots_mut,
    },
};
use ena::unify::InPlaceUnificationTable;

#[derive(Default, Debug, Clone)]
pub struct RemoveExtraRegistersPass {}

impl Pass for RemoveExtraRegistersPass {
    fn description() -> &'static str {
        "Remove extra registers"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        // Create a union table
        let mut table = InPlaceUnificationTable::<EnaKey>::new();
        // Map each register ID to a EnaKey
        let reg_map: HashMap<RegisterId<_>, EnaKey> = input
            .symtab
            .iter_reg()
            .map(|(reg, _)| (reg, table.new_key(())))
            .collect();
        let inv_map: HashMap<EnaKey, RegisterId<_>> =
            reg_map.iter().map(|(&op, &key)| (key, op)).collect();
        // Loop over the assignments in the opcodes. And for each assignment,
        // union the arguments in the table.
        for lop in &input.ops {
            if let OpCode::Assign(assign) = &lop.op {
                if let (Some(lhs_reg), Some(rhs_reg)) = (assign.lhs.reg(), assign.rhs.reg()) {
                    let lhs_key = reg_map[&lhs_reg];
                    let rhs_key = reg_map[&rhs_reg];
                    table.union(lhs_key, rhs_key);
                }
            }
        }
        // Next, rewrite the ops, where for each operand, we take the root of the unify tree
        visit_object_slots_mut(&mut input, |_sense, slot| {
            if let Some(reg) = slot.reg() {
                let key = reg_map[&reg];
                let root = table.find(key);
                let replacement = Slot::Register(inv_map[&root]);
                *slot = replacement;
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
