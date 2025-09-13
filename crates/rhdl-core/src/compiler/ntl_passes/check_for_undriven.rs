use std::collections::HashSet;

use crate::{
    RHDLError,
    {
        common::symtab::RegisterId,
        error::rhdl_error,
        ntl::{Object, error::NetListError, spec::WireKind, visit::visit_wires},
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
        let mut written_set: HashSet<RegisterId<WireKind>> = HashSet::default();
        for lop in &input.ops {
            visit_wires(&lop.op, |sense, op| {
                if sense.is_write() {
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
                if sense.is_read() {
                    if let Some(reg) = op.reg() {
                        if !written_set.contains(&reg) {
                            err = Some(NetListError {
                                cause: crate::ntl::error::NetListICE::UndrivenNetlistNode,
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
