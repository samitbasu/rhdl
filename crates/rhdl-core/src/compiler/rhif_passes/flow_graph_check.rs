use std::sync::Arc;

use crate::{
    RHDLError,
    rhif::{Object, flow_graph::build_flow_graph},
};

use super::pass::Pass;

pub struct FlowGraphCheckPass;

impl Pass for FlowGraphCheckPass {
    fn run(input: Object) -> Result<Object, RHDLError> {
        let _ = build_flow_graph(Arc::new(input.clone()))?;
        Ok(input)
    }
    fn description() -> &'static str {
        "Check that flow graph can be built."
    }
}
