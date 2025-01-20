use crate::rhdl_core::{
    flow_graph::passes::{
        //        check_for_undriven::CheckForUndrivenPass,
        constant_buffer_elimination::ConstantBufferEliminationPass,
        remove_orphan_constants::RemoveOrphanConstantsPass,
    },
    FlowGraph, RHDLError,
};

use super::passes::{
    check_for_logic_loops::CheckForLogicLoops, check_for_undriven::CheckForUndrivenPass,
    constant_propagation::ConstantPropagationPass,
    lower_any_with_single_argument::LowerAnyWithSingleArgument,
    lower_case_to_select::LowerCaseToSelectPass, lower_select_to_buffer::LowerSelectToBufferPass,
    lower_select_with_identical_args::LowerSelectWithIdenticalArgs, pass::Pass,
    remove_and_with_constant::RemoveAndWithConstantPass,
    remove_hardwired_selects::RemoveHardwiredSelectsPass,
    remove_or_with_constant::RemoveOrWithConstantPass, remove_unused_buffers::RemoveUnusedBuffers,
    remove_useless_selects::RemoveUselessSelectsPass,
    remove_zeros_from_any::RemoveZerosFromAnyPass,
};

pub fn optimize_flow_graph(mut flow_graph: FlowGraph) -> Result<FlowGraph, RHDLError> {
    loop {
        let hash_id = flow_graph.hash_value();
        flow_graph = ConstantBufferEliminationPass::run(flow_graph)?;
        flow_graph = RemoveOrphanConstantsPass::run(flow_graph)?;
        flow_graph = RemoveHardwiredSelectsPass::run(flow_graph)?;
        flow_graph = RemoveUnusedBuffers::run(flow_graph)?;
        flow_graph = ConstantPropagationPass::run(flow_graph)?;
        flow_graph = RemoveUselessSelectsPass::run(flow_graph)?;
        flow_graph = LowerCaseToSelectPass::run(flow_graph)?;
        flow_graph = RemoveOrWithConstantPass::run(flow_graph)?;
        flow_graph = RemoveAndWithConstantPass::run(flow_graph)?;
        flow_graph = RemoveZerosFromAnyPass::run(flow_graph)?;
        flow_graph = LowerAnyWithSingleArgument::run(flow_graph)?;
        flow_graph = LowerSelectWithIdenticalArgs::run(flow_graph)?;
        flow_graph = LowerSelectToBufferPass::run(flow_graph)?;
        if flow_graph.hash_value() == hash_id {
            break;
        }
    }
    flow_graph = CheckForUndrivenPass::run(flow_graph)?;
    flow_graph = CheckForLogicLoops::run(flow_graph)?;
    Ok(flow_graph)
}
