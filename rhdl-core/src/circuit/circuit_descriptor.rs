use super::circuit_impl::Circuit;
use crate::flow_graph::edge_kind::EdgeKind;
use crate::flow_graph::flow_graph_impl::{FlowGraph, FlowIx};
use crate::rtl::object::RegisterKind;
use crate::types::digital::Digital;
use crate::types::path::{bit_range, Path};
use crate::types::tristate::Tristate;
use crate::{build_rtl_flow_graph, compile_design, CompilationMode, RHDLError, Synchronous};
use crate::{util::hash_id, Kind};
use std::collections::{BTreeMap, HashMap};

// A few notes on the circuit descriptor struct
// The idea here is to capture the details on the circuit in such
// a way that it can be manipulated at run time.  This means that
// information encoded in the type system must be lifted into the
// runtime description.  And the repository for that information
// is the CircuitDescriptor struct.  We cannot, for example, iterate
// over the types that make up our children.
#[derive(Clone, Debug)]
pub struct CircuitDescriptor {
    pub unique_name: String,
    pub input_kind: Kind,
    pub output_kind: Kind,
    pub d_kind: Kind,
    pub q_kind: Kind,
    pub num_tristate: usize,
    pub tristate_offset_in_parent: usize,
    pub flow_graph: FlowGraph,
    pub children: BTreeMap<String, CircuitDescriptor>,
}

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
// put there, the better.  Thus, we build the flow graph when we build the
// circuit descriptor.  Because of the need to have the children present, we rely
// on them being passed in as a map, with their descriptors already built.
pub fn build_descriptor<C: Circuit>(
    circuit: &C,
    children: BTreeMap<String, CircuitDescriptor>,
) -> Result<CircuitDescriptor, RHDLError> {
    let module = compile_design::<C::Update>(CompilationMode::Asynchronous)?;
    let update_flow_graph = build_rtl_flow_graph(&module);
    let mut fg = FlowGraph::default();
    let output_kind: RegisterKind = C::O::static_kind().into();
    let d_kind: RegisterKind = C::D::static_kind().into();
    let q_kind: RegisterKind = C::Q::static_kind().into();
    let input_kind: RegisterKind = C::I::static_kind().into();
    // Merge in the flow graph of the update function (and keep it's remap)
    let update_remap = fg.merge(&update_flow_graph);
    let remap_bits = |x: &[FlowIx]| x.iter().map(|y| update_remap[y]).collect::<Vec<_>>();
    // We need an input buffer
    let input_buffer = fg.buffer(input_kind, "i", None);
    let input_from_update = remap_bits(&update_flow_graph.inputs[0]);
    // Link the input to its buffer
    for (input, input_buffer) in input_from_update.iter().zip(input_buffer.iter()) {
        fg.edge(*input_buffer, *input, EdgeKind::Arg(0));
    }
    let update_q_input = remap_bits(&update_flow_graph.inputs[1]);
    // We need an output buffer, but will need to split the output from the update map into its two constituent components.
    let update_output = remap_bits(&update_flow_graph.output);
    let output_buffer_location = update_flow_graph.graph[update_flow_graph.output[0]].location;
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
    for (buffer, q) in q_buffer.iter().zip(&update_q_input) {
        fg.edge(*buffer, *q, EdgeKind::Arg(0));
    }
    // Create the inputs for the children by splitting bits off of the d_index
    for (child_name, child_descriptor) in &children {
        // Compute the bit range for this child's input based on its name
        let child_path = Path::default().field(child_name);
        let (output_bit_range, _) = bit_range(C::D::static_kind(), &child_path)?;
        let (input_bit_range, _) = bit_range(C::Q::static_kind(), &child_path)?;
        let child_flow_graph = &child_descriptor.flow_graph;
        let child_remap = fg.merge(child_flow_graph);
        let remap_child = |x: &[FlowIx]| x.iter().map(|y| child_remap[y]).collect::<Vec<_>>();
        let child_inputs = remap_child(&child_flow_graph.inputs[0]);
        let child_output = remap_child(&child_flow_graph.output);
        let mut d_iter = circuit_d_buffer.iter().skip(output_bit_range.start);
        for (child_input, d_index) in child_inputs.iter().zip(&mut d_iter) {
            fg.edge(*d_index, *child_input, EdgeKind::Arg(0));
        }
        let mut q_iter = q_buffer.iter().skip(input_bit_range.start);
        for (child_output, q_index) in child_output.iter().zip(&mut q_iter) {
            fg.edge(*child_output, *q_index, EdgeKind::Arg(0));
        }
    }
    fg.inputs = vec![input_buffer];
    fg.output = circuit_output_buffer;
    Ok(CircuitDescriptor {
        unique_name: format!(
            "{}_{:x}",
            circuit.name(),
            hash_id(std::any::TypeId::of::<C>())
        ),
        input_kind: C::I::static_kind(),
        output_kind: C::O::static_kind(),
        d_kind: C::D::static_kind(),
        q_kind: C::Q::static_kind(),
        num_tristate: C::Z::N,
        flow_graph: fg,
        tristate_offset_in_parent: 0,
        children,
    })
}

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
pub fn build_synchronous_descriptor<C: Synchronous>(
    circuit: &C,
    children: BTreeMap<String, CircuitDescriptor>,
) -> Result<CircuitDescriptor, RHDLError> {
    let module = compile_design::<C::Update>(CompilationMode::Synchronous)?;
    let update_flow_graph = build_rtl_flow_graph(&module);
    // A synchronous flow graph has separate clock and
    // reset inputs, but these don't really factor into
    // data flow, since the assumption is that all elements
    // of a synchronous circuit are clocked and reset together.
    // However, we fan out the clock and reset to each of the children
    // in so they can use them.
    let mut fg = FlowGraph::default();
    // This is the kind of output of the update kernel - it must be equal to
    // (Update::O, Update::D)
    // The update_fg will have 3 arguments (rst,i,q) and 2 outputs (o,d)
    let output_kind: RegisterKind = C::O::static_kind().into();
    let d_kind: RegisterKind = C::D::static_kind().into();
    let q_kind: RegisterKind = C::Q::static_kind().into();
    let input_kind: RegisterKind = C::I::static_kind().into();
    // Merge in the flow graph of the update function (and keep it's remap)
    let update_remap = fg.merge(&update_flow_graph);
    let remap_bits = |x: &[FlowIx]| x.iter().map(|y| update_remap[y]).collect::<Vec<_>>();
    // We need a cr buffer - it is mandatory.
    let cr_buffer = fg.buffer(RegisterKind::Unsigned(2), "cr", None);
    // We also need an input buffer
    let input_buffer = fg.buffer(input_kind, "i", None);
    let cr_for_update = remap_bits(&update_flow_graph.inputs[0]);
    // We need an input buffer (if we have any inputs)
    let input_from_update = remap_bits(&update_flow_graph.inputs[1]);
    // Link the input and reset to their respective buffers
    for (reset, reset_buffer) in cr_for_update.iter().zip(cr_buffer.iter()) {
        fg.edge(*reset_buffer, *reset, EdgeKind::Arg(0));
    }
    for (input, input_buffer) in input_from_update.iter().zip(input_buffer.iter()) {
        fg.edge(*input_buffer, *input, EdgeKind::Arg(0));
    }
    let update_q_input = remap_bits(&update_flow_graph.inputs[2]);
    // We need an output buffer, but we will need to split the output from the update map into it's two constituent components.
    let update_output = remap_bits(&update_flow_graph.output);
    let output_buffer_location = update_flow_graph.graph[update_flow_graph.output[0]].location;
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
    for (child_name, child_descriptor) in &children {
        // Compute the bit range for this child's input based on its name
        // The tuple index of .1 is to get the D element of the output from the kernel
        let child_path = Path::default().field(child_name);
        let (output_bit_range, _) = bit_range(C::D::static_kind(), &child_path)?;
        let (input_bit_range, _) = bit_range(C::Q::static_kind(), &child_path)?;
        let child_flow_graph = &child_descriptor.flow_graph;
        let child_remap = fg.merge(&child_flow_graph);
        let remap_child = |x: &[FlowIx]| x.iter().map(|y| child_remap[y]).collect::<Vec<_>>();
        let child_inputs = remap_child(&child_flow_graph.inputs[1]);
        let child_output = remap_child(&child_flow_graph.output);
        let mut d_iter = circuit_d_buffer.iter().skip(output_bit_range.start);
        for (child_input, d_index) in child_inputs.iter().zip(&mut d_iter) {
            fg.edge(*d_index, *child_input, EdgeKind::Arg(0));
        }
        let mut q_iter = q_buffer.iter().skip(input_bit_range.start);
        for (child_output, q_index) in child_output.iter().zip(&mut q_iter) {
            fg.edge(*child_output, *q_index, EdgeKind::Arg(0));
        }
        // Connect the cr lines
        let cr_line = remap_child(&child_flow_graph.inputs[0]);
        for (reset_buffer, reset_line) in cr_buffer.iter().zip(cr_line.iter()) {
            fg.edge(*reset_buffer, *reset_line, EdgeKind::Arg(0));
        }
    }
    fg.inputs = vec![cr_buffer, input_buffer];
    fg.output = circuit_output_buffer;
    Ok(CircuitDescriptor {
        unique_name: format!(
            "{}_{:x}",
            circuit.name(),
            hash_id(std::any::TypeId::of::<C>())
        ),
        input_kind: C::I::static_kind(),
        output_kind: C::O::static_kind(),
        d_kind: C::D::static_kind(),
        q_kind: C::Q::static_kind(),
        num_tristate: C::Z::N,
        tristate_offset_in_parent: 0,
        children,
        flow_graph: fg,
    })
}
