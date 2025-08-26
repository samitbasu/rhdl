use std::collections::HashMap;

use crate::{
    prelude::RHDLError,
    rhdl_core::{
        common::symtab::RegisterId,
        ntl::{
            object::Object,
            spec::{Assign, OpCode, Wire, WireKind},
            visit::visit_object_wires_mut,
        },
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
        let mut constant_set: HashMap<RegisterId<WireKind>, Wire> = HashMap::default();
        for op in &input.ops {
            if let OpCode::Assign(assign) = &op.op {
                if let Some(reg_id) = assign.lhs.reg() {
                    if let Some(_) = input.bitx(assign.rhs) {
                        constant_set.insert(reg_id, assign.rhs);
                    }
                }
            }
        }
        // Rewrite ops to use constants where possible
        visit_object_wires_mut(&mut input, |sense, op| {
            if sense.is_read() {
                if let Some(reg) = op.reg() {
                    if let Some(lit) = constant_set.get(&reg) {
                        *op = *lit;
                    }
                }
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
