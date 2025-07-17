use crate::rhdl_core::{
    common::symtab::SymbolTable,
    error::RHDLError,
    rhif::{
        Object,
        spec::{Assign, OpCode, Slot},
    },
    types::path::Path,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct PrecomputeDiscriminantPass {}

impl Pass for PrecomputeDiscriminantPass {
    fn description() -> &'static str {
        "Precompute discriminants of literals"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let (mut literals, registers) = std::mem::take(&mut input.symtab).into_parts();
        let mut ops = std::mem::take(&mut input.ops);
        for lop in ops.iter_mut() {
            if let OpCode::Index(index) = &lop.op {
                if index.path == Path::default().discriminant() {
                    let kind = match index.arg {
                        Slot::Register(rid) => registers[rid].0,
                        Slot::Literal(lid) => literals[lid].0.kind,
                    };
                    if !kind.is_enum() {
                        lop.op = OpCode::Assign(Assign {
                            lhs: index.lhs,
                            rhs: index.arg,
                        });
                    } else if let Slot::Literal(lit_id) = index.arg {
                        let (literal_value, loc) = &literals[&lit_id];
                        let discriminant = literal_value.discriminant()?;
                        // Get a new literal slot for the discriminant
                        let discriminant_id = literals.push((discriminant, loc.clone()));
                        let discriminant_slot = Slot::Literal(discriminant_id);
                        lop.op = OpCode::Assign(Assign {
                            lhs: index.lhs,
                            rhs: discriminant_slot,
                        });
                    }
                }
            }
        }
        input.symtab = SymbolTable::from_parts(literals, registers);
        input.ops = ops;
        Ok(input)
    }
}
