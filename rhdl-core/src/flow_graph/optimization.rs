use crate::{
    flow_graph::passes::{
        //        check_for_undriven::CheckForUndrivenPass,
        constant_buffer_elimination::ConstantBufferEliminationPass,
        remove_orphan_constants::RemoveOrphanConstantsPass,
    },
    FlowGraph, RHDLError,
};

use super::passes::{
    constant_propagation::ConstantPropagationPass, pass::Pass,
    remove_hardwired_selects::RemoveHardwiredSelectsPass,
    remove_unused_buffers::RemoveUnusedBuffers,
};

pub fn optimize_flow_graph(mut flow_graph: FlowGraph) -> Result<FlowGraph, RHDLError> {
    loop {
        let hash_id = flow_graph.hash_value();
        flow_graph = ConstantBufferEliminationPass::run(flow_graph)?;
        flow_graph = RemoveOrphanConstantsPass::run(flow_graph)?;
        flow_graph = RemoveHardwiredSelectsPass::run(flow_graph)?;
        flow_graph = RemoveUnusedBuffers::run(flow_graph)?;
        flow_graph = ConstantPropagationPass::run(flow_graph)?;
        if flow_graph.hash_value() == hash_id {
            break;
        }
    }
    //    flow_graph = CheckForUndrivenPass::run(flow_graph)?;
    Ok(flow_graph)
}
