use std::iter::once;

use petgraph::visit::EdgeRef;

use crate::{
    ast::source_location::SourceLocation,
    error::rhdl_error,
    flow_graph::{edge_kind::EdgeKind, error::FlowGraphError},
    hdl::ast::{
        always, binary, concatenate, constant, continuous_assignment, declaration, id,
        if_statement, index_bit, non_blocking_assignment, port, select, unary, unsigned_width,
        Declaration, Direction, Events, Expression, HDLKind, Module, Statement,
    },
    util::delim_list_optional_strings,
    FlowGraph, RHDLError,
};

use super::{
    component::{self, Component, ComponentKind},
    error::FlowGraphICE,
    flow_graph_impl::{FlowIx, GraphType},
};

// Generate a register declaration for the given component.

fn node(index: FlowIx) -> String {
    format!("node_{}", index.index())
}

fn nodes(indices: Vec<FlowIx>) -> Vec<Box<Expression>> {
    indices.into_iter().map(|ndx| id(&node(ndx))).collect()
}

fn generate_wire_declaration(index: FlowIx, component: &Component) -> Option<Declaration> {
    if component.width == 0 {
        None
    } else {
        Some(declaration(
            HDLKind::Wire,
            &node(index),
            unsigned_width(component.width),
            Some(format!("{:?}", component)),
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

fn make_select_assign_statement(index: FlowIx, graph: &FlowGraph) -> Result<Statement, RHDLError> {
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
    Ok(continuous_assignment(
        &node(index),
        select(
            id(&node(control_node)),
            id(&node(true_node)),
            id(&node(false_node)),
        ),
    ))
}

fn make_dff_input_assign_statement(
    index: FlowIx,
    graph: &FlowGraph,
) -> Result<Statement, RHDLError> {
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
            continuous_assignment(&node(index), index_bit(&node(data_edge.source()), *bit))
        }
        _ => continuous_assignment(&node(index), id(&node(data_edge.source()))),
    })
}

fn make_buffer_assign_statement(index: FlowIx, graph: &FlowGraph) -> Result<Statement, RHDLError> {
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
        Ok(continuous_assignment(
            &node(index),
            index_bit(&format!("arg_{}", arg_index), bit_pos),
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
                continuous_assignment(&node(index), index_bit(&node(parent.source()), *bit))
            }
            _ => continuous_assignment(&node(index), id(&node(parent.source()))),
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
            let bit = width - 1 - bit;
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

fn make_binary_assign_statement(index: FlowIx, graph: &FlowGraph) -> Result<Statement, RHDLError> {
    let component = &graph.graph[index];
    let ComponentKind::Binary(op) = &component.kind else {
        return Err(raise_ice(
            FlowGraphICE::ExpectedBinaryComponent,
            graph,
            component.location,
        ));
    };
    let arg0 = concatenate(nodes(collect_argument(index, 0, component.width, graph)?));
    let arg1 = concatenate(nodes(collect_argument(index, 1, component.width, graph)?));
    Ok(continuous_assignment(
        &node(index),
        binary(op.op, arg0, arg1),
    ))
}

fn make_unary_assign_statement(index: FlowIx, graph: &FlowGraph) -> Result<Statement, RHDLError> {
    let component = &graph.graph[index];
    let ComponentKind::Unary(op) = &component.kind else {
        return Err(raise_ice(
            FlowGraphICE::ExpectedUnaryComponent,
            graph,
            component.location,
        ));
    };
    let arg = nodes(collect_argument(index, 0, component.width, graph)?);
    Ok(continuous_assignment(
        &node(index),
        unary(op.op, concatenate(arg)),
    ))
}

fn generate_assign_statement(
    index: FlowIx,
    graph: &FlowGraph,
) -> Result<Option<Statement>, RHDLError> {
    let component = &graph.graph[index];
    match &component.kind {
        ComponentKind::Constant(value) => {
            Ok(Some(continuous_assignment(&node(index), constant(*value))))
        }
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

pub(crate) fn generate_hdl(module_name: &str, fg: &FlowGraph) -> Result<Module, RHDLError> {
    dbg!(&fg.inputs);
    let ports = fg
        .inputs
        .iter()
        .enumerate()
        .flat_map(|(ndx, x)| {
            (!x.is_empty()).then(|| {
                port(
                    &format!("arg_{ndx}"),
                    Direction::Input,
                    HDLKind::Wire,
                    unsigned_width(x.len()),
                )
            })
        })
        .chain(once(port(
            "out",
            Direction::Output,
            HDLKind::Wire,
            unsigned_width(fg.output.len()),
        )))
        .collect();
    let mut declarations = fg
        .graph
        .node_indices()
        .filter_map(|ndx| generate_wire_declaration(ndx, &fg.graph[ndx]))
        .collect::<Vec<_>>();
    let mut statements = fg
        .graph
        .node_indices()
        .filter_map(|ndx| generate_assign_statement(ndx, fg).transpose())
        .collect::<Result<Vec<_>, _>>()?;
    for (ndx, dff) in fg.dffs.iter().enumerate() {
        let reg_name = format!("dff_{}", ndx);
        // To create a DFF, we need a registers to hold the output of the DFF
        declarations.push(declaration(
            HDLKind::Reg,
            &reg_name,
            unsigned_width(1),
            Some(format!("{:?}", fg.graph[dff.output])),
        ));
        // Get the clock wire
        let clock = fg
            .graph
            .edges_directed(dff.input, petgraph::Direction::Incoming)
            .find_map(|edge| match edge.weight() {
                EdgeKind::Clock => Some(edge.source()),
                _ => None,
            })
            .ok_or(raise_ice(
                FlowGraphICE::ClockNotFound,
                fg,
                fg.graph[dff.input].location,
            ))?;
        let reset = fg
            .graph
            .edges_directed(dff.input, petgraph::Direction::Incoming)
            .find_map(|edge| match edge.weight() {
                EdgeKind::Reset => Some(edge.source()),
                _ => None,
            })
            .ok_or(raise_ice(
                FlowGraphICE::ResetNotFound,
                fg,
                fg.graph[dff.input].location,
            ))?;
        // Create an always block for the DFF
        let block = always(
            vec![Events::Posedge(node(clock))],
            vec![if_statement(
                id(&node(reset)),
                vec![non_blocking_assignment(
                    &reg_name,
                    constant(dff.reset_value),
                )],
                vec![non_blocking_assignment(&reg_name, id(&node(dff.input)))],
            )],
        );
        statements.push(block);
        statements.push(continuous_assignment(&node(dff.output), id(&reg_name)));
    }
    let output_bits = concatenate(
        fg.output
            .iter()
            .rev()
            .map(|ndx| id(&node(*ndx)))
            .collect::<Vec<_>>(),
    );
    statements.push(continuous_assignment("out", output_bits));
    Ok(Module {
        name: module_name.to_string(),
        ports,
        declarations,
        statements,
        ..Default::default()
    })
}
