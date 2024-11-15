use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveZerosFromAnyPass {}

fn get_any_with_


impl Pass for RemoveZerosFromAnyPass {
    fn run(mut input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        let mut graph = std::mem::take(&mut input.graph);
        let candidates = graph.node_indices().flat_map(|node| {

        })
    }
}
