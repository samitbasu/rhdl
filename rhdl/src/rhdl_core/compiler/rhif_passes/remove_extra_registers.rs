use std::collections::HashMap;

use super::pass::Pass;
use crate::rhdl_core::{
    common::symtab::RegisterId,
    error::RHDLError,
    rhif::{
        Object,
        spec::{Assign, OpCode, Slot},
        visit::visit_object_slots_mut,
    },
};
use ena::unify::{InPlaceUnificationTable, UnifyKey};

#[derive(Default, Debug, Clone)]
pub struct RemoveExtraRegistersPass {}

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
struct RegisterKey(u32);

impl UnifyKey for RegisterKey {
    type Value = ();

    fn index(&self) -> u32 {
        self.0
    }

    fn from_index(u: u32) -> Self {
        Self(u)
    }

    fn tag() -> &'static str {
        "RegisterKey"
    }
}

impl Pass for RemoveExtraRegistersPass {
    fn description() -> &'static str {
        "Remove extra registers"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        // Create a union table
        let mut table = InPlaceUnificationTable::<RegisterKey>::new();
        // Map each register ID to a RegisterKey
        let reg_map: HashMap<RegisterId, RegisterKey> = input
            .symtab
            .iter_reg()
            .map(|(reg, _)| (reg, table.new_key(())))
            .collect();
        let inv_map: HashMap<RegisterKey, RegisterId> =
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
        let symbols = std::mem::take(&mut input.symbols);
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
        input.symbols = symbols;
        Ok(input)
    }
}

fn merge_names(a: Option<&String>, b: Option<&String>) -> Option<String> {
    match (a, b) {
        (None, None) => None,
        (Some(a), None) => Some(a.clone()),
        (None, Some(b)) => Some(b.clone()),
        (Some(a), Some(b)) if a == b => Some(a.clone()),
        (Some(a), Some(b)) => Some(format!("{b}_then_{a}")),
    }
}
