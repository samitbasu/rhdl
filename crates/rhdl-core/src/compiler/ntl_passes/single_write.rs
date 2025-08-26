use std::collections::HashSet;

use crate::{
    prelude::RHDLError,
    rhdl_core::{
        common::symtab::RegisterId,
        compiler::mir::error::ICE,
        ntl::{Object, spec::WireKind, visit::visit_wires},
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct SingleRegisterWrite {}

impl Pass for SingleRegisterWrite {
    fn description() -> &'static str {
        "Check every register is written once"
    }
    fn run(input: Object) -> Result<Object, RHDLError> {
        let mut written_set: HashSet<RegisterId<WireKind>> = HashSet::default();
        let mut err = None;
        for lop in &input.ops {
            visit_wires(&lop.op, |sense, op| {
                if sense.is_write() {
                    if let Some(reg) = op.reg() {
                        if !written_set.insert(reg) {
                            if err.is_none() {
                                err = Some(Self::raise_ice(
                                    &input,
                                    ICE::MultipleWritesToRegister { op: *op },
                                    lop.loc,
                                ));
                            }
                        }
                    }
                }
            });
            if let Some(err) = err {
                return Err(err);
            }
        }
        Ok(input)
    }
}
