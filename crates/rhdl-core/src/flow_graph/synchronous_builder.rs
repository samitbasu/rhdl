use std::sync::Arc;

use crate::{
    ClockReset, Descriptor, Digital, Kind, RHDLError, ScopedName, SyncKind, Synchronous,
    SynchronousIO,
    flow_graph::{
        ChildOutputKernelQ, FlowGraph, KernelDToChildInput, builder::Builder, errors::FlowGraphICE,
        rhif_builder::build_flow_graph,
    },
    types::path::{Path, PathExt},
};

pub fn build_synchronous_flowgraph<C: Synchronous>(
    scoped_name: &ScopedName,
    kernel: &crate::rtl::Object,
    children: &[Descriptor<SyncKind>],
) -> Result<FlowGraph, RHDLError> {
    let mut builder = Builder::new(&scoped_name.to_string());
    let kernel_fg = build_flow_graph(Arc::clone(&kernel.rhif))?;
    let cr_kind: Kind = ClockReset::static_kind();
    let input_kind: Kind = <<C as SynchronousIO>::I as Digital>::static_kind();
    let output_kind: Kind = <<C as SynchronousIO>::O as Digital>::static_kind();
    builder.add_input_port(cr_kind, 0);
    builder.add_input_port(input_kind, 1);
    builder.add_output_port(output_kind);
    let kernel_map = builder.import(&kernel_fg);
    for path in cr_kind.all_leafs() {
        let input_port = builder.get_input_port(0, &path)?;
        let kernel_input = kernel_map[&kernel_fg.input_port(0, &path)?];
        builder.add_edge(
            input_port,
            kernel_input,
            crate::flow_graph::EdgeKind::ClockResetToKernel,
        );
    }
    for path in input_kind.all_leafs() {
        let input_port = builder.get_input_port(1, &path)?;
        let kernel_input = kernel_map[&kernel_fg.input_port(1, &path)?];
        builder.add_edge(
            input_port,
            kernel_input,
            crate::flow_graph::EdgeKind::InputToKernel,
        );
    }
    for path in output_kind.all_leafs() {
        let output_port = builder.get_output_port(&path)?;
        let kernel_output =
            kernel_map[&kernel_fg.output_port(&Path::default().tuple_index(0).join(&path))?];
        builder.add_edge(
            kernel_output,
            output_port,
            crate::flow_graph::EdgeKind::KernelToOutput,
        );
    }
    for child_descriptor in children {
        let child_name = child_descriptor.name.last().unwrap();
        let Some(child_flow_graph) = child_descriptor.flow_graph.as_ref() else {
            return Err(FlowGraphICE::MissingChildFlowGraph(child_descriptor.name.clone()).into());
        };
        let child_remap = builder.import(child_flow_graph);
        // Connect the clock/reset input of the circuit to the clock/reset input of the child
        for path in cr_kind.all_leafs() {
            let circuit_cr_port = builder.get_input_port(0, &path)?;
            let child_input_port = child_remap[&child_flow_graph.input_port(0, &path)?];
            builder.add_edge(
                circuit_cr_port,
                child_input_port,
                crate::flow_graph::EdgeKind::InputForwardToChild,
            );
        }
        // Connect all of the input ports of the child to the kernel D output.
        // This is output.1.<child_name>.path
        let child_input_kind = child_descriptor.input_kind;
        for path in child_input_kind.all_leafs() {
            // This is the port on the child flow graph
            let child_input_port = child_flow_graph.input_port(1, &path)?;
            // We need the corresponding port on the kernel output
            let kernel_output_port = kernel_map[&kernel_fg
                .output_port(&Path::default().tuple_index(1).field(child_name).join(&path))?];
            builder.add_edge(
                kernel_output_port,
                child_remap[&child_input_port],
                crate::flow_graph::EdgeKind::KernelDToChildInput(KernelDToChildInput {
                    parent: std::any::type_name::<C>(),
                    child: child_descriptor.type_name,
                    path: path.clone(),
                }),
            );
        }
        // Connect the childs output ports to the kernel input ports
        // This is input[2].<child_name>.path
        let child_output_kind = child_descriptor.output_kind;
        for path in child_output_kind.all_leafs() {
            let child_output_port = child_flow_graph.output_port(&path)?;
            let kernel_input_port = kernel_map
                [&kernel_fg.input_port(2, &Path::default().field(child_name).join(&path))?];
            builder.add_edge(
                child_remap[&child_output_port],
                kernel_input_port,
                crate::flow_graph::EdgeKind::ChildOutputKernelQ(ChildOutputKernelQ {
                    parent: std::any::type_name::<C>(),
                    child: child_descriptor.type_name,
                    path: path.clone(),
                }),
            );
        }
    }
    Ok(builder.build())
}
