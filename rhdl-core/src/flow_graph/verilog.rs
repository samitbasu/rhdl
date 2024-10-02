use std::iter::once;

use petgraph::visit::EdgeRef;

use crate::{
    ast::source_location::SourceLocation,
    error::rhdl_error,
    flow_graph::{edge_kind::EdgeKind, error::FlowGraphError},
    util::delim_list_optional_strings,
    verilog::ast::Module,
    FlowGraph, RHDLError,
};

use super::{
    component::{self, Component, ComponentKind},
    error::FlowGraphICE,
    flow_graph_impl::{FlowIx, GraphType},
};

// Generate a register declaration for the given component.

fn generate_register_declaration(index: FlowIx, component: &Component) -> Option<String> {
    if component.width == 0 {
        None
    } else {
        Some(format!(
            "reg [{}:0] node_{}; // {:?}",
            component.width.saturating_sub(1),
            index.index(),
            component
        ))
    }
}

fn raise_ice(
    cause: FlowGraphICE,
    graph: &FlowGraph,
    location: Option<SourceLocation>,
) -> RHDLError {
    rhdl_error(FlowGraphError {
        cause,
        src: graph.code.source(),
        elements: location
            .map(|loc| graph.code.span(loc).into())
            .into_iter()
            .collect(),
    })
}

fn make_select_assign_statement(index: FlowIx, graph: &FlowGraph) -> Result<String, RHDLError> {
    let component = &graph.graph[index];
    let ComponentKind::Select = &component.kind else {
        return Err(raise_ice(
            FlowGraphICE::ExpectedSelectComponent,
            graph,
            component.location,
        ));
    };
    let control_node = graph
        .graph
        .edges_directed(index, petgraph::Direction::Incoming)
        .find_map(|edge| match edge.weight() {
            EdgeKind::Selector(0) => Some(edge.source()),
            _ => None,
        })
        .ok_or(raise_ice(
            FlowGraphICE::SelectControlNodeNotFound,
            graph,
            component.location,
        ))?;
    let true_node = graph
        .graph
        .edges_directed(index, petgraph::Direction::Incoming)
        .find_map(|edge| match edge.weight() {
            EdgeKind::True => Some(edge.source()),
            _ => None,
        })
        .ok_or(raise_ice(
            FlowGraphICE::SelectTrueNodeNotFound,
            graph,
            component.location,
        ))?;
    let false_node = graph
        .graph
        .edges_directed(index, petgraph::Direction::Incoming)
        .find_map(|edge| match edge.weight() {
            EdgeKind::False => Some(edge.source()),
            _ => None,
        })
        .ok_or(raise_ice(
            FlowGraphICE::SelectFalseNodeNotFound,
            graph,
            component.location,
        ))?;
    Ok(format!(
        "assign node_{} = node_{} ? node_{} : node_{};",
        index.index(),
        control_node.index(),
        true_node.index(),
        false_node.index()
    ))
}

fn make_dff_input_assign_statement(index: FlowIx, graph: &FlowGraph) -> Result<String, RHDLError> {
    let component = &graph.graph[index];
    let ComponentKind::DFFInput(_) = &component.kind else {
        return Err(raise_ice(
            FlowGraphICE::ExpectedDFFComponent,
            graph,
            component.location,
        ));
    };
    let data_edge = graph
        .graph
        .edges_directed(index, petgraph::Direction::Incoming)
        .find(|edge| {
            matches!(
                edge.weight(),
                EdgeKind::OutputBit(_) | EdgeKind::ArgBit(_, _) | EdgeKind::Arg(_)
            )
        })
        .ok_or(raise_ice(
            FlowGraphICE::DFFInputDriverNotFound,
            graph,
            component.location,
        ))?;
    Ok(match data_edge.weight() {
        EdgeKind::OutputBit(bit) => {
            format!(
                "assign node_{} = node_{}[{}];",
                index.index(),
                data_edge.source().index(),
                bit
            )
        }
        _ => {
            format!(
                "assign node_{} = node_{};",
                index.index(),
                data_edge.source().index()
            )
        }
    })
}

fn make_buffer_assign_statement(index: FlowIx, graph: &FlowGraph) -> Result<String, RHDLError> {
    let component = &graph.graph[index];
    let ComponentKind::Buffer(name) = &component.kind else {
        return Err(raise_ice(
            FlowGraphICE::ExpectedBufferComponent,
            graph,
            component.location,
        ));
    };
    // Check for an input buffer case
    if let Some((arg_index, arg_bits)) = graph
        .inputs
        .iter()
        .enumerate()
        .find(|(_, x)| x.contains(&index))
    {
        let bit_pos = arg_bits.iter().position(|x| x == &index).unwrap();
        Ok(format!(
            "assign node_{} = arg_{arg_index}[{bit_pos}];",
            index.index(),
        ))
    } else {
        let parent = graph
            .graph
            .edges_directed(index, petgraph::Direction::Incoming)
            .next()
            .ok_or(raise_ice(
                FlowGraphICE::BufferParentNotFound,
                graph,
                component.location,
            ))?;
        Ok(match parent.weight() {
            EdgeKind::OutputBit(bit) => {
                format!(
                    "assign node_{} = node_{}[{}];",
                    index.index(),
                    parent.source().index(),
                    bit
                )
            }
            _ => {
                format!(
                    "assign node_{} = node_{};",
                    index.index(),
                    parent.source().index()
                )
            }
        })
    }
}

fn collect_argument(
    node: FlowIx,
    index: usize,
    width: usize,
    graph: &FlowGraph,
) -> Result<Vec<FlowIx>, RHDLError> {
    let component = &graph.graph[node];
    let arg_incoming = graph
        .graph
        .edges_directed(node, petgraph::Direction::Incoming)
        .filter_map(|edge| match edge.weight() {
            EdgeKind::ArgBit(ndx, bit) if *ndx == index => Some((*bit, edge.source())),
            EdgeKind::Arg(ndx) if *ndx == index => Some((0, edge.source())),
            _ => None,
        })
        .collect::<Vec<_>>();
    (0..width)
        .map(|bit| {
            arg_incoming
                .iter()
                .find_map(|(b, ndx)| if *b == bit { Some(*ndx) } else { None })
                .ok_or(raise_ice(
                    FlowGraphICE::MissingArgument { index, bit },
                    graph,
                    component.location,
                ))
        })
        .collect()
}

fn make_binary_assign_statement(index: FlowIx, graph: &FlowGraph) -> Result<String, RHDLError> {
    let component = &graph.graph[index];
    let ComponentKind::Binary(op) = &component.kind else {
        return Err(raise_ice(
            FlowGraphICE::ExpectedBinaryComponent,
            graph,
            component.location,
        ));
    };
    let op = ""; // FIXME op.op.verilog_binop();
    let arg0 = collect_argument(index, 0, component.width, graph)?
        .iter()
        .rev()
        .map(|x| format!("node_{}", x.index()))
        .collect::<Vec<_>>()
        .join(", ");
    let arg1 = collect_argument(index, 1, component.width, graph)?
        .iter()
        .rev()
        .map(|x| format!("node_{}", x.index()))
        .collect::<Vec<_>>()
        .join(", ");
    Ok(format!(
        "assign node_{} = {{ {arg0} }} {} {{ {arg1} }};",
        index.index(),
        op
    ))
}

fn make_unary_assign_statement(index: FlowIx, graph: &FlowGraph) -> Result<String, RHDLError> {
    let component = &graph.graph[index];
    let ComponentKind::Unary(op) = &component.kind else {
        return Err(raise_ice(
            FlowGraphICE::ExpectedUnaryComponent,
            graph,
            component.location,
        ));
    };
    let op = op.op.verilog_unop();
    let arg = collect_argument(index, 0, component.width, graph)?
        .iter()
        .rev()
        .map(|x| format!("node_{}", x.index()))
        .collect::<Vec<_>>()
        .join(", ");
    Ok(format!(
        "assign node_{} = {} ( {{ {arg} }} );",
        index.index(),
        op
    ))
}

fn generate_assign_statement(
    index: FlowIx,
    graph: &FlowGraph,
) -> Result<Option<String>, RHDLError> {
    let component = &graph.graph[index];
    match &component.kind {
        ComponentKind::Constant(value) => Ok(Some(format!(
            "assign node_{} = 1'b{};",
            index.index(),
            if *value { 1 } else { 0 }
        ))),
        ComponentKind::Buffer(_) => Ok(Some(make_buffer_assign_statement(index, graph)?)),
        ComponentKind::Select => Ok(Some(make_select_assign_statement(index, graph)?)),
        ComponentKind::Binary(_) => Ok(Some(make_binary_assign_statement(index, graph)?)),
        ComponentKind::Unary(_) => Ok(Some(make_unary_assign_statement(index, graph)?)),
        ComponentKind::DFFInput(_) => Ok(Some(make_dff_input_assign_statement(index, graph)?)),
        ComponentKind::DFFOutput(_) => Ok(None),
        _ => todo!(
            "No assign implementation for {:?} index {}",
            component,
            index.index()
        ),
    }
}

pub fn generate_verilog(module_name: &str, fg: &FlowGraph) -> Result<Module, RHDLError> {
    dbg!(&fg.inputs);
    let args = fg
        .inputs
        .iter()
        .enumerate()
        .map(|(ndx, x)| {
            if x.is_empty() {
                None
            } else {
                Some(format!(
                    "input wire [{}:0] arg_{}",
                    x.len().saturating_sub(1),
                    ndx
                ))
            }
        })
        .chain(once(Some(format!(
            "output wire [{}:0] out",
            fg.output.len().saturating_sub(1)
        ))))
        .collect::<Vec<_>>();
    let args = delim_list_optional_strings(&args, ", ");
    let module_decl = format!("module {module_name}({});", args);
    let reg_decls = fg
        .graph
        .node_indices()
        .filter_map(|ndx| generate_register_declaration(ndx, &fg.graph[ndx]))
        .collect::<Vec<_>>();
    let assign_stmts = fg
        .graph
        .node_indices()
        .filter_map(|ndx| generate_assign_statement(ndx, fg).transpose())
        .collect::<Result<Vec<_>, _>>()?;
    //    let dff_stmts = generate_dff_statements(fg)?;
    // FIXME
    let foo = format!(
        "{module_decl}\n{reg_decl}\n{assign_stmts}\nendmodule",
        reg_decl = reg_decls.join(""),
        assign_stmts = assign_stmts.join("\n"),
    );
    Ok(Module::default())
}
