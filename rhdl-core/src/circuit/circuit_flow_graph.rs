use crate::{
    flow_graph::{edge_kind::EdgeKind, flow_graph_impl::FlowIx},
    rtl::object::RegisterKind,
    CircuitDescriptor, FlowGraph, RHDLError,
};

// Create a flow graph of an arbitrary circuit.  The model is
//
//          +-----------------+
//          |                 |
//   *in ---> In              Out >-----*out
//          |                 |
//     +--> Q                 D >-+
//     |    |                 |   |
//     |    +-----------------+   |
//     |                          |
//     +--< Out child 0      In <-+
//     |                          |
//     +--< Out child 1      In <-+
//
// Note - we don't want to build this in the proc-macro since the less logic we
// put there, the better.
fn build_asynchronous_flow_graph_internal(
    descriptor: &CircuitDescriptor,
) -> Result<FlowGraph, RHDLError> {
    let mut fg = FlowGraph::default();
    let output_kind: RegisterKind = (&descriptor.output_kind).into();
    let d_kind: RegisterKind = (&descriptor.d_kind).into();
    let q_kind: RegisterKind = (&descriptor.q_kind).into();
    let input_kind: RegisterKind = (&descriptor.input_kind).into();
    // Merge in the flow graph of the update function (and keep it's remap)
    let update_remap = fg.merge(&descriptor.update_flow_graph);
    let remap_bits = |x: &[FlowIx]| x.iter().map(|y| update_remap[y]).collect::<Vec<_>>();
    // We need an input buffer
    let input_buffer = fg.buffer(input_kind, "i", None);
    let input_from_update = remap_bits(&descriptor.update_flow_graph.inputs[0]);
    // Link the input to its buffer
    for (input, input_buffer) in input_from_update.iter().zip(input_buffer.iter()) {
        fg.edge(*input_buffer, *input, EdgeKind::Arg(0));
    }
    let update_q_input = remap_bits(&descriptor.update_flow_graph.inputs[1]);
    let update_output = remap_bits(&descriptor.update_flow_graph.output);
    let output_buffer_location =
        descriptor.update_flow_graph.graph[descriptor.update_flow_graph.output[0]].location;
    // This is the circuit output buffer (contains the circuit output)
    let circuit_output_buffer = fg.buffer(output_kind, "o", output_buffer_location);
    let mut update_output_bits = update_output.iter();
    // Assign the output buffer to the output of the update function
    for (circuit, output) in circuit_output_buffer.iter().zip(&mut update_output_bits) {
        fg.edge(*output, *circuit, EdgeKind::Arg(0));
    }
    // Create a buffer to hold the "D" output of the update function
    let circuit_d_buffer = fg.buffer(d_kind, "d", output_buffer_location);

    todo!()
}
