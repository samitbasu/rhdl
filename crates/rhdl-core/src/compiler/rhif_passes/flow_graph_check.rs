use internment::Intern;

use crate::{
    RHDLError,
    rhif::{Object, flow_graph::build_flow_graph},
};

use super::pass::Pass;

pub struct FlowGraphCheckPass;

impl Pass for FlowGraphCheckPass {
    fn run(input: Object) -> Result<Object, RHDLError> {
        let interned = Intern::new(input);
        let _ = build_flow_graph(interned)?;
        Ok((*interned).clone())
    }
    fn description() -> &'static str {
        "Check that flow graph can be built."
    }
}
