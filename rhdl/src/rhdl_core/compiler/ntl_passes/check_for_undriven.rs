use std::collections::HashSet;

use crate::{
    prelude::RHDLError,
    rhdl_core::{
        error::rhdl_error,
        ntl::{
            error::NetListError,
            spec::RegisterId,
            visit::{visit_wires, Sense},
            Object,
        },
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct CheckForUndriven {}

impl Pass for CheckForUndriven {
    fn description() -> &'static str {
        "Check For Undriven values"
    }
    fn run(input: Object) -> Result<Object, RHDLError> {
        let mut written_set: HashSet<RegisterId> = HashSet::default();
        for lop in &input.ops {
            visit_wires(&lop.op, |sense, op| {
                if sense == Sense::Write {
                    if let Some(reg) = op.reg() {
                        written_set.insert(reg);
                    }
                }
            })
        }
        written_set.extend(input.inputs.iter().flatten().copied());
        for lop in &input.ops {
            let mut err = None;
            visit_wires(&lop.op, |sense, op| {
                if sense == Sense::Read {
                    if let Some(reg) = op.reg() {
                        if !written_set.contains(&reg) {
                            err = Some(NetListError {
                                cause:
                                    crate::rhdl_core::ntl::error::NetListICE::UndrivenNetlistNode,
                                src: input.code.source(),
                                elements: lop
                                    .loc
                                    .iter()
                                    .map(|&loc| input.code.span(loc).into())
                                    .collect(),
                            });
                        }
                    }
                }
            });
            if let Some(err) = err {
                return Err(rhdl_error(err));
            }
        }
        Ok(input)
    }
}
