use anyhow::{bail, Result};
use petgraph::{stable_graph::NodeIndex, Direction};
use rhdl_core::{
    diagnostic::dfg::DFG,
    path::{bit_range, Path},
    Kind,
};

// Given a DFG, a node in that DFG, and a path that into the input of that node,
// Collect a graph of all possible sources of that signal.
pub fn trace(dfg: &DFG, node: NodeIndex<u32>, path: &Path) -> Result<()> {
    // First, iterate through the edges that end on node
    // and find the one that covers the given path.
    let component = dfg.graph[node];
    let Some(covering_edge) = dfg
        .graph
        .edges_directed(node, Direction::Incoming)
        .find(|edge| {
            path_covers(component.input.clone(), &edge.weight().dest, path).unwrap_or_default()
        })
    else {
        bail!("No edge covers the given path");
    };
    let link = covering_edge.weight();
    let path = path_transmute(path, &link.dest, &link.src);
}

fn path_covers(kind: Kind, sub_path: &Path, parent_path: &Path) -> Result<bool> {
    let sub_bit_range = bit_range(kind.clone(), &sub_path)?;
    let parent_bit_range = bit_range(kind, &parent_path)?;
    Ok(parent_bit_range.0.contains(&sub_bit_range.0.start)
        && parent_bit_range.0.contains(&sub_bit_range.0.end))
}

fn path_transmute(original: &Path, prefix_to_remove: &Path, prefix_to_add: &Path) -> Path {
    let path_with_prefix_stripped = original
        .iter()
        .cloned()
        .skip(prefix_to_remove.len())
        .collect::<Path>();
    let mut new_path = prefix_to_add.clone();
    new_path.join(&path_with_prefix_stripped);
    new_path
}
