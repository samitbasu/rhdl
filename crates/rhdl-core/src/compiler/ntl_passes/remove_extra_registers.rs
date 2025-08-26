use std::collections::{HashMap, HashSet};

use ena::unify::InPlaceUnificationTable;

use crate::{
    prelude::RHDLError,
    rhdl_core::{
        common::{symtab::RegisterId, unify_key::EnaKey},
        compiler::ntl_passes::pass::Pass,
        ntl::{
            object::Object,
            spec::{Assign, OpCode, Wire, WireKind},
            visit::{visit_object_wires, visit_object_wires_mut},
        },
    },
};

#[derive(Default, Debug, Clone)]
pub struct RemoveExtraRegistersPass {}

impl Pass for RemoveExtraRegistersPass {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        // Create a union table
        let mut table = InPlaceUnificationTable::<EnaKey>::default();
        let mut reg_set: HashSet<RegisterId<WireKind>> = HashSet::default();
        visit_object_wires(&input, |_sense, wire| {
            if let Some(reg) = wire.reg() {
                reg_set.insert(reg);
            }
        });
        // Map each operand to a key in the table
        let reg_map: HashMap<RegisterId<WireKind>, EnaKey> = reg_set
            .into_iter()
            .map(|op| {
                let key = table.new_key(());
                (op, key)
            })
            .collect();
        let inv_map: HashMap<EnaKey, RegisterId<WireKind>> =
            reg_map.iter().map(|(&op, &key)| (key, op)).collect();
        // Loop over the operands, and for every operand that is an assignment,
        // union the arguments in the table
        log::info!("Remove extra registers: {input:?}");
        for op in &input.ops {
            if let OpCode::Assign(assign) = &op.op {
                if let (Some(lhs_reg), Some(rhs_reg)) = (assign.lhs.reg(), assign.rhs.reg()) {
                    let lhs_key = reg_map[&lhs_reg];
                    let rhs_key = reg_map[&rhs_reg];
                    table.union(lhs_key, rhs_key);
                }
            }
        }
        // Next, rewrite the ops, where for each operand, we take the root of the unify tree
        visit_object_wires_mut(&mut input, |_sense, wire| {
            if let Some(reg) = wire.reg() {
                let key = reg_map[&reg];
                let root = table.find(key);
                let replacement = Wire::Register(inv_map[&root]);
                *wire = replacement;
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
        "Remove Extra Registers"
    }
}
