use std::sync::Arc;

use crate::{RHDLError, flow_graph::rhif_builder::build_flow_graph, rhif::Object};

use super::pass::Pass;

pub struct FlowGraphCheckPass;

impl Pass for FlowGraphCheckPass {
    fn run(input: Object) -> Result<Object, RHDLError> {
        let fg = build_flow_graph(Arc::new(input.clone()))?;
        if petgraph::algo::is_cyclic_directed(&fg.graph) {
            panic!("Flow graph is cyclic");
        }
        Ok(input)
    }
    fn description() -> &'static str {
        "Check that flow graph can be built."
    }
}
