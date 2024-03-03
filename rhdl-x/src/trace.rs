use std::collections::{HashMap, HashSet};

use anyhow::{bail, Result};
use petgraph::{stable_graph::NodeIndex, visit::EdgeRef, Direction};
use rhdl_core::{
    diagnostic::dfg::{Component, ComponentKind, Link, DFG},
    path::{bit_range, Path},
    rhif::spec::AluBinary,
    Kind,
};

#[derive(Default, Debug)]
pub struct DFGSubset {
    nodes: HashSet<NodeIndex<u32>>,
    links: Vec<(NodeIndex<u32>, NodeIndex<u32>, Link)>,
}

pub fn subgraph(dfg: &DFG, subset: &DFGSubset) -> DFG {
    let mut relocation = HashMap::new();
    let mut new_graph = DFG::default();
    for node in subset.nodes.iter() {
        let new_node = new_graph.graph.add_node(dfg.graph[*node].clone());
        relocation.insert(*node, new_node);
    }
    for (src, dest, link) in subset.links.iter() {
        let new_src = relocation[src];
        let new_dest = relocation[dest];
        new_graph.graph.add_edge(new_src, new_dest, link.clone());
    }
    new_graph
}

// Given a DFG, a node in that DFG, and a path that into the input of that node,
// Collect a graph of all possible sources of that signal.
pub fn trace(
    dfg: &DFG,
    node: NodeIndex<u32>,
    orig_path: &Path,
    subset: &mut DFGSubset,
) -> Result<()> {
    // First, iterate through the edges that end on node
    // and find the one that covers the given path.
    let component = &dfg.graph[node];
    eprintln!("trace component: {} path {orig_path}", component);
    assert!(is_path_valid(component.input.clone(), orig_path));
    subset.nodes.insert(node);
    let Some(covering_edge) = dfg
        .graph
        .edges_directed(node, Direction::Incoming)
        .find(|edge| {
            path_covers(component.input.clone(), orig_path, &edge.weight().dest).unwrap_or_default()
        })
    else {
        bail!("No edge covers the given path {orig_path} for node {component:?}");
    };
    let link = covering_edge.weight();
    let path = path_transmute(orig_path, &link.dest, &link.src);
    // Get the node at the start of the covering edge
    let source_node_ix = covering_edge.source();
    subset.nodes.insert(source_node_ix);
    let source_node = &dfg.graph[source_node_ix];
    eprintln!("parent path is {source_node}{path}");
    assert!(is_path_valid(source_node.output.clone(), &path));
    subset.links.push((
        source_node_ix,
        node,
        Link {
            src: path.clone(),
            dest: orig_path.clone(),
        },
    ));
    let input_paths = map_outputs_to_input(source_node, &path);
    eprintln!("input paths {input_paths:?}");
    for input_path in input_paths {
        trace(dfg, source_node_ix, &input_path.input, subset)?;
    }
    Ok(())
}

fn is_path_valid(kind: Kind, path: &Path) -> bool {
    bit_range(kind, path).is_ok()
}

fn path_covers(kind: Kind, sub_path: &Path, parent_path: &Path) -> Result<bool> {
    let (sub_bit_range, sub_kind) = bit_range(kind.clone(), &sub_path)?;
    let (parent_bit_range, parent_kind) = bit_range(kind, &parent_path)?;
    eprintln!("Path test {sub_path} {sub_bit_range:?} {sub_kind} {parent_path} {parent_bit_range:?} {parent_kind}");
    Ok((parent_bit_range.start <= sub_bit_range.start)
        && (parent_bit_range.end >= sub_bit_range.end))
}

fn path_transmute(original: &Path, prefix_to_remove: &Path, prefix_to_add: &Path) -> Path {
    let path_with_prefix_stripped = original
        .iter()
        .skip(prefix_to_remove.len())
        .cloned()
        .collect::<Path>();
    let mut new_path = prefix_to_add.clone();
    new_path.join(&path_with_prefix_stripped)
}

#[derive(Debug)]
struct InputPathForGivenOutput {
    output: Path,
    input: Path,
    conditional: bool,
}

fn map_outputs_to_input(component: &Component, path: &Path) -> Vec<InputPathForGivenOutput> {
    match &component.kind {
        ComponentKind::Binary(bin) => map_outputs_to_input_binary(bin, path),
        ComponentKind::Array => todo!(),
        ComponentKind::Select => map_outputs_to_input_select(path),
        ComponentKind::Buffer(_) => map_outputs_to_input_buffer(path),
        ComponentKind::DFF => vec![],
        ComponentKind::Case(len) => map_outputs_to_input_case(*len, path),
        ComponentKind::Index(index_path) => map_outputs_to_input_index(index_path, path),
        ComponentKind::Constant => vec![],
        ComponentKind::Tuple => map_outputs_to_input_tuple(component, path),
        ComponentKind::Splice(splice_path) => {
            map_outputs_to_input_splice(component, splice_path, path)
        }
        _ => todo!(
            "map_outputs_to_input for {:?} not implemented yet",
            component.kind
        ),
    }
}

fn map_outputs_to_input_splice(
    component: &Component,
    slice_path: &Path,
    path: &Path,
) -> Vec<InputPathForGivenOutput> {
    assert!(!slice_path.any_dynamic());
    eprintln!("map_outputs_to_input_splice {component} {slice_path} {path}");
    // Check if the spliced bits include the requested path
    let ret = if path_covers(component.output.clone(), path, slice_path).unwrap() {
        eprintln!("path is covered - use substitute input");
        // If so, then we divert to the substutute field on the input
        let substitute_path = path_transmute(path, slice_path, &Path::default().field("subst"));
        vec![InputPathForGivenOutput {
            output: path.clone(),
            input: substitute_path,
            conditional: false,
        }]
    } else {
        eprintln!("path is not covered - use original input");
        // Take from the original input
        vec![InputPathForGivenOutput {
            output: path.clone(),
            input: Path::default().field("orig").join(path),
            conditional: false,
        }]
    };
    assert!(is_path_valid(component.input.clone(), &ret[0].input));
    ret
}

fn map_outputs_to_input_tuple(component: &Component, path: &Path) -> Vec<InputPathForGivenOutput> {
    vec![InputPathForGivenOutput {
        output: path.clone(),
        input: path.clone(),
        conditional: false,
    }]
}

fn map_outputs_to_input_index(index_path: &Path, path: &Path) -> Vec<InputPathForGivenOutput> {
    // What happens if this includes dynamic indexing?
    assert!(!index_path.any_dynamic());
    vec![InputPathForGivenOutput {
        output: path.clone(),
        input: index_path.clone().join(path),
        conditional: false,
    }]
}

fn map_outputs_to_input_binary(bin: &AluBinary, path: &Path) -> Vec<InputPathForGivenOutput> {
    vec![
        InputPathForGivenOutput {
            output: path.clone(),
            input: Path::default().index(0).join(path),
            conditional: false,
        },
        InputPathForGivenOutput {
            output: path.clone(),
            input: Path::default().index(1).join(path),
            conditional: false,
        },
    ]
}

fn map_outputs_to_input_select(path: &Path) -> Vec<InputPathForGivenOutput> {
    vec![
        InputPathForGivenOutput {
            output: path.clone(),
            input: Path::default().field("true_value").join(path),
            conditional: true,
        },
        InputPathForGivenOutput {
            output: path.clone(),
            input: Path::default().field("false_value").join(path),
            conditional: true,
        },
    ]
}

fn map_outputs_to_input_buffer(path: &Path) -> Vec<InputPathForGivenOutput> {
    vec![InputPathForGivenOutput {
        output: path.clone(),
        input: Path::default().join(path),
        conditional: false,
    }]
}

fn map_outputs_to_input_case(table_len: usize, path: &Path) -> Vec<InputPathForGivenOutput> {
    (0..table_len)
        .map(|i| InputPathForGivenOutput {
            output: path.clone(),
            input: Path::default().field("table").index(i).join(path),
            conditional: true,
        })
        .collect()
}
