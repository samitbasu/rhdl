use crate::{
    flow_graph::{
        edge_kind::EdgeKind,
        flow_graph_impl::{FlowGraph, FlowIx},
        passes::{check_for_undriven::CheckForUndrivenPass, pass::Pass},
    },
    rtl::object::RegisterKind,
    types::path::{bit_range, Path},
    CircuitDescriptor, RHDLError,
};

// Create a flow graph of the circuit.  It is modified by adding
// a Q buffer and a D buffer.
//
//        +-----------------------------+
//        | +--------------------+      |
//        | |                    |      |
//   *rst +-> Reset              |      |
//          |                    |      |
//   *in ---> In                Out >------*out
//          |        update      |      |
//     +--> Q                   D >-+   |
//     |    |                    |  |   |
//     |    +--------------------+  |   |
//     |                            |   |
//     |                            |   |
//     |                      rst <-----+
//     +--< Out   child 0      In <-+   |
//     |                            |   |
//     |                      rst <-----+
//     +--< Out    child 1     In <-+
// Note - we don't want to build this in the proc-macro since the less logic we
// put there, the better.
fn build_synchronous_flow_graph_internal(
    descriptor: &CircuitDescriptor,
) -> Result<FlowGraph, RHDLError> {
    // A synchronous flow graph has separate clock and
    // reset inputs, but these don't really factor into
    // data flow, since the assumption is that all elements
    // of a synchronous circuit are clocked and reset together.
    let mut fg = FlowGraph::default();
    // This is the kind of output of the update kernel - it must be equal to
    // (Update::O, Update::D)
    // The update_fg will have 3 arguments (rst,i,q) and 2 outputs (o,d)
    let output_kind: RegisterKind = (&descriptor.output_kind).into();
    let d_kind: RegisterKind = (&descriptor.d_kind).into();
    let q_kind: RegisterKind = (&descriptor.q_kind).into();
    let input_kind: RegisterKind = (&descriptor.input_kind).into();
    // Merge in the flow graph of the update function (and keep it's remap)
    let update_remap = fg.merge(&descriptor.update_flow_graph);
    let remap_bits = |x: &[FlowIx]| x.iter().map(|y| update_remap[y]).collect::<Vec<_>>();
    // We need a reset buffer - it is mandatory.
    let reset_buffer = fg.buffer(RegisterKind::Unsigned(1), "reset", None);
    // We also need an input buffer
    let input_buffer = fg.buffer(input_kind, "i", None);
    let reset_from_update = remap_bits(&descriptor.update_flow_graph.inputs[0]);
    // We need an input buffer (if we have any inputs)
    let input_from_update = remap_bits(&descriptor.update_flow_graph.inputs[1]);
    // Link the input and reset to their respective buffers
    for (reset, reset_buffer) in reset_from_update.iter().zip(reset_buffer.iter()) {
        fg.edge(*reset_buffer, *reset, EdgeKind::Arg(0));
    }
    for (input, input_buffer) in input_from_update.iter().zip(input_buffer.iter()) {
        fg.edge(*input_buffer, *input, EdgeKind::Arg(0));
    }
    let update_q_input = remap_bits(&descriptor.update_flow_graph.inputs[2]);
    // We need an output buffer, but we will need to split the output from the update map into it's two constituent components.
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
    for (d, output) in circuit_d_buffer.iter().zip(&mut update_output_bits) {
        fg.edge(*output, *d, EdgeKind::Arg(0));
    }
    // Create a buffer to hold the "Q" input of the update function
    let q_buffer = fg.buffer(q_kind, "q", output_buffer_location);
    // Wire that buffer to the input of the update function
    for (buffer, q) in q_buffer.iter().zip(&update_q_input) {
        fg.edge(*buffer, *q, EdgeKind::Arg(0));
    }
    // Create the inputs for the children by splitting bits off of the d_index
    for (child_name, child_descriptor) in &descriptor.children {
        // Compute the bit range for this child's input based on its name
        // The tuple index of .1 is to get the D element of the output from the kernel
        let output_path = Path::default().field(child_name);
        let (output_bit_range, _) = bit_range(descriptor.d_kind.clone(), &output_path)?;
        let input_path = Path::default().field(child_name);
        let (input_bit_range, _) = bit_range(descriptor.q_kind.clone(), &input_path)?;
        // TODO - get the bit ranges
        eprintln!("Output_kind {:?}", output_kind);
        eprintln!("Child: {}", child_name);
        let child_flow_graph = build_synchronous_flow_graph_internal(child_descriptor)?;
        let child_remap = fg.merge(&child_flow_graph);
        let remap_child = |x: &[FlowIx]| x.iter().map(|y| child_remap[y]).collect::<Vec<_>>();
        let child_inputs = remap_child(&child_flow_graph.inputs[1]);
        eprintln!("Child inputs: {:?}", child_inputs);
        for input in child_inputs.iter() {
            eprintln!("Input: {:?}", fg.graph[*input]);
        }
        let child_output = remap_child(&child_flow_graph.output);
        let mut d_iter = circuit_d_buffer.iter().skip(output_bit_range.start);
        for (child_input, d_index) in child_inputs.iter().zip(&mut d_iter) {
            fg.edge(*d_index, *child_input, EdgeKind::Arg(0));
        }
        let mut q_iter = q_buffer.iter().skip(input_bit_range.start);
        for (child_output, q_index) in child_output.iter().zip(&mut q_iter) {
            fg.edge(*child_output, *q_index, EdgeKind::Arg(0));
        }
        // Connect the reset line
        let reset_line = remap_child(&child_flow_graph.inputs[0]);
        for (reset_buffer, reset_line) in reset_buffer.iter().zip(reset_line.iter()) {
            fg.edge(*reset_buffer, *reset_line, EdgeKind::Arg(0));
        }
    }
    fg.inputs = vec![reset_buffer, input_buffer];
    fg.output = circuit_output_buffer;
    Ok(fg)
}

pub fn build_synchronous_flow_graph(
    descriptor: &CircuitDescriptor,
) -> Result<FlowGraph, RHDLError> {
    let fg = build_synchronous_flow_graph_internal(descriptor)?.sealed();
    let fg = CheckForUndrivenPass::run(fg)?;
    Ok(fg)
}
