use crate::{
    flow_graph::{
        component::{ComponentKind, Constant, Index, Splice},
        edge_kind::EdgeKind,
        flow_graph_impl::FlowGraph,
    },
    rtl::object::{BitString, RegisterKind},
    types::path::{bit_range, Path},
    CircuitDescriptor,
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
fn build_synchronous_flow_graph_internal(descriptor: &CircuitDescriptor) -> FlowGraph {
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
    // Merge in the flow graph of the update function (and keep it's remap)
    let update_remap = fg.merge(&descriptor.update_flow_graph);
    // We need a reset buffer - it is mandatory.
    let reset_buffer = descriptor.update_flow_graph.inputs[0].map(|node| update_remap[&node]);
    // We need an input buffer (if we have any inputs)
    let input_buffer = descriptor.update_flow_graph.inputs[1].map(|node| update_remap[&node]);
    let update_q_input = descriptor.update_flow_graph.inputs[2].map(|node| update_remap[&node]);
    // We need an output buffer, but we will need to split the output from the update map into it's two constituent components.
    let update_output = descriptor.update_flow_graph.output;
    let output_buffer_location =
        descriptor.update_flow_graph.graph[descriptor.update_flow_graph.output].location;
    // This is the circuit output buffer (contains the circuit output)
    let circuit_output_buffer = fg.buffer(output_kind, "o", output_buffer_location);
    // We need to split the output of the update function into (o, i_c0, i_c1, i_c2,...)
    let output_index = fg.new_component_with_optional_location(
        ComponentKind::Index(Index {
            bit_range: 0..output_kind.len(),
        }),
        output_buffer_location,
    );
    // Connect this component to the update function's output
    fg.edge(output_index, update_output, EdgeKind::Arg(0));
    let d_index = if !d_kind.is_empty() {
        let d_index = fg.new_component_with_optional_location(
            ComponentKind::Index(Index {
                bit_range: output_kind.len()..(output_kind.len() + d_kind.len()),
            }),
            output_buffer_location,
        );
        fg.edge(d_index, update_output, EdgeKind::Arg(0));
        Some(d_index)
    } else {
        None
    };
    let mut q_buffer = if descriptor.q_kind.is_empty() {
        None
    } else {
        let len = descriptor.q_kind.bits();
        let bs = if descriptor.q_kind.is_signed() {
            BitString::Signed(vec![false; len])
        } else {
            BitString::Unsigned(vec![false; len])
        };
        Some(
            fg.new_component_with_optional_location(ComponentKind::Constant(Constant { bs }), None),
        )
    };
    // Create the inputs for the children by splitting bits off of the d_index
    for (child_name, child_descriptor) in &descriptor.children {
        // Compute the bit range for this child's input based on it's name
        // The tuple index of .1 is to get the D element of the output from the kernel
        let output_path = Path::default().field(child_name);
        eprintln!("Output_kind {:?}", output_kind);
        let (child_input_range, _) = bit_range(descriptor.d_kind.clone(), &output_path).unwrap();
        let (child_output_range, _) = bit_range(descriptor.q_kind.clone(), &output_path).unwrap();
        let child_flow_graph = build_synchronous_flow_graph_internal(child_descriptor);
        let child_remap = fg.merge(&child_flow_graph);
        if !child_input_range.is_empty() {
            let child_index = fg.new_component_with_optional_location(
                ComponentKind::Index(Index {
                    bit_range: child_input_range,
                }),
                output_buffer_location,
            );
            fg.edge(child_index, d_index.unwrap(), EdgeKind::Arg(0));
            fg.edge(
                child_remap[&child_flow_graph.inputs[1].unwrap()],
                child_index,
                EdgeKind::Arg(0),
            );
        }
        // Connect the reset line
        if let Some(child_reset) = child_flow_graph.inputs[0] {
            fg.edge(
                child_remap[&child_reset],
                reset_buffer.unwrap(),
                EdgeKind::Arg(0),
            );
        }
        if !child_output_range.is_empty() {
            // Splice the child output into the q_buffer
            let new_q = fg.new_component_with_optional_location(
                ComponentKind::Splice(Splice {
                    bit_range: child_output_range,
                }),
                None,
            );
            fg.edge(new_q, q_buffer.unwrap(), EdgeKind::Arg(0));
            fg.edge(
                new_q,
                child_remap[&child_flow_graph.output],
                EdgeKind::Splice,
            );
            q_buffer = Some(new_q);
        }
    }
    if let Some(q_buffer) = q_buffer {
        // Add a named buffer to make it easier to understand
        let q_named_buffer = fg.buffer(q_kind, "q", None);
        fg.edge(q_named_buffer, q_buffer, EdgeKind::Arg(0));
        fg.edge(update_q_input.unwrap(), q_named_buffer, EdgeKind::Arg(0));
    }
    fg.edge(circuit_output_buffer, output_index, EdgeKind::Arg(0));
    fg.inputs = vec![reset_buffer, input_buffer];
    fg.output = circuit_output_buffer;
    fg
}

pub fn build_synchronous_flow_graph(descriptor: &CircuitDescriptor) -> FlowGraph {
    let internal_fg = build_synchronous_flow_graph_internal(descriptor);
    // Create a new, top level FG with sources for the inputs and sinks for the
    // outputs.
    let mut fg = FlowGraph::default();
    let remap = fg.merge(&internal_fg);
    let timing_start = fg.new_component_with_optional_location(ComponentKind::TimingStart, None);
    let timing_end = fg.new_component_with_optional_location(ComponentKind::TimingEnd, None);
    // Create sources for all of the inputs of the internal flow graph
    internal_fg.inputs.iter().flatten().for_each(|input| {
        let component = &internal_fg.graph[*input];
        let ComponentKind::Buffer(buffer) = &component.kind else {
            panic!("flow graph inputs must be buffers")
        };
        let input_source = fg.source(buffer.kind, &buffer.name, component.location);
        fg.edge(remap[input], input_source, EdgeKind::Arg(0));
    });
    // Add a source for the output
    let internal_output = &internal_fg.graph[internal_fg.output];
    let ComponentKind::Buffer(buffer) = &internal_output.kind else {
        panic!("flow graph outputs must be buffers")
    };
    let output_sink = fg.sink(buffer.kind, &buffer.name, internal_output.location);
    fg.edge(output_sink, remap[&internal_fg.output], EdgeKind::Arg(0));
    // Next, iterate over all nodes of the graph, and connect each source to the timing start
    // and each sink to the timing end.
    for node in fg.graph.node_indices() {
        let component = &fg.graph[node];
        match &component.kind {
            ComponentKind::Source(_) => fg.edge(node, timing_start, EdgeKind::Virtual),
            ComponentKind::Sink(_) => fg.edge(timing_end, node, EdgeKind::Virtual),
            _ => (),
        }
    }
    fg.inputs = vec![Some(timing_start)];
    fg.output = timing_end;
    fg
}
