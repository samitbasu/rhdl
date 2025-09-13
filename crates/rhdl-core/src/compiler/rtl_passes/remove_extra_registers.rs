use std::collections::HashMap;

use crate::{
    RHDLError,
    common::{symtab::RegisterId, unify_key::EnaKey},
    rtl::{
        Object,
        spec::{Assign, OpCode, Operand},
        visit::visit_object_operands_mut,
    },
};
use ena::unify::InPlaceUnificationTable;

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveExtraRegistersPass {}

impl Pass for RemoveExtraRegistersPass {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        // Create a union table
        let mut table = InPlaceUnificationTable::<EnaKey>::new();
        // Map each Register ID to an EnaKey
        let reg_map: HashMap<RegisterId<_>, EnaKey> = input
            .symtab
            .iter_reg()
            .map(|(reg, _)| (reg, table.new_key(())))
            .collect();
        let inv_map: HashMap<EnaKey, RegisterId<_>> =
            reg_map.iter().map(|(&reg, &key)| (key, reg)).collect();
        // Loop over the assignment op codes, and union the arguments in the table
        for lop in &input.ops {
            if let OpCode::Assign(Assign { lhs, rhs }) = &lop.op {
                if let (Some(lhs_reg), Some(rhs_reg)) = (lhs.reg(), rhs.reg()) {
                    let lhs_key = reg_map[&lhs_reg];
                    let rhs_key = reg_map[&rhs_reg];
                    table.union(lhs_key, rhs_key);
                }
            }
        }
        // Next, rewrite the ops, where for each operand, we take the root of the unify tree
        visit_object_operands_mut(&mut input, |_sense, operand| {
            if let Some(reg) = operand.reg() {
                let key = reg_map[&reg];
                let root = table.find(key);
                let replacement = Operand::Register(inv_map[&root]);
                *operand = replacement;
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
    fn description() -> &'static str {
        "Remove extra registers"
    }
}
