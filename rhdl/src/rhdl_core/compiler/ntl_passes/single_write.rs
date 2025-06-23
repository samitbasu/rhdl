use std::collections::HashSet;

use crate::{
    prelude::RHDLError,
    rhdl_core::{
        compiler::mir::error::ICE,
        ntl::{
            spec::RegisterId,
            visit::{visit_operands, Sense},
            Object,
        },
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
        let mut written_set: HashSet<RegisterId> = HashSet::default();
        written_set.extend(input.inputs.iter().flatten());
        for lop in &input.ops {
            let mut err = None;
            visit_operands(&lop.op, |sense, op| {
                if sense == Sense::Write {
                    if let Some(reg) = op.reg() {
                        if !written_set.insert(reg) {
                            err = Some(Self::raise_ice(
                                &input,
                                ICE::MultipleWritesToRegister { op: *op },
                                lop.loc,
                            ));
                        }
                    }
                }
            })
        }
        Ok(input)
    }
}
