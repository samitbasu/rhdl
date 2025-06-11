use std::collections::{HashMap, HashSet};

use ena::unify::{EqUnifyValue, InPlaceUnificationTable, UnifyKey};

use crate::{
    prelude::RHDLError,
    rhdl_core::{
        compiler::ntl_passes::pass::Pass,
        ntl::{
            object::{LocatedOpCode, Object},
            remap::remap_operands,
            spec::{Assign, OpCode, Operand, RegisterId},
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
        let ops = std::mem::take(&mut input.ops);
        // Create a union table
        let mut table = InPlaceUnificationTable::<RegisterKey>::default();
        let mut reg_set: HashSet<RegisterId> = HashSet::default();
        // Define each of the operands
        let ops = ops
            .into_iter()
            .map(|lop| LocatedOpCode {
                loc: lop.loc,
                op: remap_operands(lop.op, |op| {
                    if let Some(reg) = op.reg() {
                        reg_set.insert(reg);
                    }
                    op
                }),
            })
            .collect::<Vec<_>>();
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
        let ops = ops
            .into_iter()
            .map(|lop| LocatedOpCode {
                loc: lop.loc,
                op: remap_operands(lop.op, |op| {
                    if let Some(reg) = op.reg() {
                        let key = reg_map[&reg];
                        let root = table.find(key);
                        Operand::Register(inv_map[&root])
                    } else {
                        op
                    }
                }),
            })
            .filter(|lop| {
                if let OpCode::Assign(Assign { lhs, rhs }) = lop.op {
                    lhs != rhs
                } else {
                    true
                }
            })
            .collect();

        input.ops = ops;
        Ok(input)
    }

    fn description() -> &'static str {
        "Remove Extra Registers"
    }
}
