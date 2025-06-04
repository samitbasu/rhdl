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

fn run_pass<P: Pass>(flow_graph: FlowGraph) -> Result<FlowGraph, RHDLError> {
    log::info!("Flow graph optimization pass {}", P::description());
    P::run(flow_graph)
}

pub fn optimize_flow_graph(mut flow_graph: FlowGraph) -> Result<FlowGraph, RHDLError> {
    loop {
        let hash_id = flow_graph.hash_value();
        flow_graph = run_pass::<ConstantBufferEliminationPass>(flow_graph)?;
        flow_graph = run_pass::<RemoveOrphanConstantsPass>(flow_graph)?;
        flow_graph = run_pass::<RemoveHardwiredSelectsPass>(flow_graph)?;
        flow_graph = run_pass::<RemoveUnusedBuffers>(flow_graph)?;
        flow_graph = run_pass::<ConstantPropagationPass>(flow_graph)?;
        flow_graph = run_pass::<RemoveUselessSelectsPass>(flow_graph)?;
        flow_graph = run_pass::<LowerCaseToSelectPass>(flow_graph)?;
        flow_graph = run_pass::<RemoveOrWithConstantPass>(flow_graph)?;
        flow_graph = run_pass::<RemoveAndWithConstantPass>(flow_graph)?;
        flow_graph = run_pass::<RemoveZerosFromAnyPass>(flow_graph)?;
        flow_graph = run_pass::<LowerAnyWithSingleArgument>(flow_graph)?;
        flow_graph = run_pass::<LowerSelectWithIdenticalArgs>(flow_graph)?;
        flow_graph = run_pass::<LowerSelectToBufferPass>(flow_graph)?;
        if flow_graph.hash_value() == hash_id {
            break;
        }
    }
    flow_graph = run_pass::<CheckForUndrivenPass>(flow_graph)?;
    flow_graph = run_pass::<CheckForLogicLoops>(flow_graph)?;
    Ok(flow_graph)
}
