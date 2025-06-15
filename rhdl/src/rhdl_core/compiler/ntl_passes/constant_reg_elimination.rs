use std::collections::HashMap;

use crate::{
    prelude::{BitX, RHDLError},
    rhdl_core::ntl::{
        object::{LocatedOpCode, Object},
        remap::{remap_operands, visit_operands_mut},
        spec::{Assign, OpCode, Operand, RegisterId},
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct ConstantRegisterElimination {}

impl Pass for ConstantRegisterElimination {
    fn description() -> &'static str {
        "Constant Buffer Elimination"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut constant_set: HashMap<RegisterId, BitX> = HashMap::default();
        let mut ops = std::mem::take(&mut input.ops);
        for op in &ops {
            if let OpCode::Assign(assign) = &op.op {
                if let Some(reg_id) = assign.lhs.reg() {
                    if let Some(bitx) = assign.rhs.bitx() {
                        constant_set.insert(reg_id, bitx);
                    }
                }
            }
        }
        // Rewrite ops to use constants where possible
        for lop in &mut ops {
            visit_operands_mut(&mut lop.op, |op| {
                if let Some(reg) = op.reg() {
                    if let Some(bitx) = constant_set.get(&reg) {
                        *op = Operand::from(*bitx);
                    }
                }
            });
        }
        ops.retain(|lop| {
            if let OpCode::Assign(Assign { lhs, rhs }) = lop.op {
                lhs != rhs
            } else {
                true
            }
        });
        input.outputs = input
            .outputs
            .into_iter()
            .map(|o| {
                if let Some(reg) = o.reg() {
                    if let Some(&bitx) = constant_set.get(&reg) {
                        return bitx.into();
                    }
                }
                o
            })
            .collect();
        input.ops = ops;
        Ok(input)
    }
}
