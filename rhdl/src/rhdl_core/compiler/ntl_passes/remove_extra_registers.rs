use std::collections::{HashMap, HashSet};

use ena::unify::{InPlaceUnificationTable, UnifyKey};

use crate::{
    prelude::RHDLError,
    rhdl_core::{
        compiler::ntl_passes::pass::Pass,
        ntl::{
            object::Object,
            spec::{Assign, OpCode, Wire, RegisterId},
            visit::{visit_wires, visit_wires_mut},
        },
    },
};

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
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut ops = std::mem::take(&mut input.ops);
        // Create a union table
        let mut table = InPlaceUnificationTable::<RegisterKey>::default();
        let mut reg_set: HashSet<RegisterId> = HashSet::default();
        for arg in &input.inputs {
            arg.iter().for_each(|&r| {
                reg_set.insert(r);
            });
        }
        for arg in input.outputs.iter().filter_map(Wire::reg) {
            reg_set.insert(arg);
        }
        // Define each of the operands
        for lop in &ops {
            visit_wires(&lop.op, |_sense, op| {
                if let Some(reg) = op.reg() {
                    reg_set.insert(reg);
                }
            });
        }
        // Map each operand to a key in the table
        let reg_map: HashMap<RegisterId, RegisterKey> = reg_set
            .into_iter()
            .map(|op| {
                let key = table.new_key(());
                (op, key)
            })
            .collect();
        let inv_map: HashMap<RegisterKey, RegisterId> =
            reg_map.iter().map(|(&op, &key)| (key, op)).collect();
        // Loop over the operands, and for every operand that is an assignment,
        // union the arguments in the table
        for op in &ops {
            if let OpCode::Assign(assign) = &op.op {
                if let (Some(lhs_reg), Some(rhs_reg)) = (assign.lhs.reg(), assign.rhs.reg()) {
                    let lhs_key = reg_map[&lhs_reg];
                    let rhs_key = reg_map[&rhs_reg];
                    table.union(lhs_key, rhs_key);
                }
            }
        }
        // Next, rewrite the ops, where for each operand, we take the root of the unify tree
        for op in &mut ops {
            visit_wires_mut(&mut op.op, |op| {
                if let Some(reg) = op.reg() {
                    let key = reg_map[&reg];
                    let root = table.find(key);
                    *op = Wire::Register(inv_map[&root])
                }
            })
        }
        ops.retain(|lop| {
            if let OpCode::Assign(Assign { lhs, rhs }) = lop.op {
                lhs != rhs
            } else {
                true
            }
        });
        // Finally, rewrite the inputs and outputs
        input.inputs.iter_mut().for_each(|v| {
            v.iter_mut().for_each(|r| {
                let key = reg_map[&*r];
                let root = table.find(key);
                *r = inv_map[&root];
            })
        });
        input.outputs.iter_mut().for_each(|o| {
            *o = if let Some(reg) = o.reg() {
                let key = reg_map[&reg];
                let root = table.find(key);
                Wire::Register(inv_map[&root])
            } else {
                *o
            }
        });
        input.ops = ops;
        Ok(input)
    }

    fn description() -> &'static str {
        "Remove Extra Registers"
    }
}
