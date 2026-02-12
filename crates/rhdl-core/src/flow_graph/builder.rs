use std::collections::HashMap;

use petgraph::graph::NodeIndex;

use crate::{
    Kind, RHDLError, TypedBits,
    flow_graph::{BufferRef, ConstantRef, EdgeKind, FlowGraph, NodeKind, NodePort},
    types::path::{Path, PathExt},
};

pub struct Builder {
    pub(crate) flow_graph: FlowGraph,
    atom_map: HashMap<NodeKind, NodeIndex>,
}

impl Builder {
    pub fn new(name: &str) -> Self {
        Self {
            flow_graph: FlowGraph {
                name: name.into(),
                graph: Default::default(),
                ports: HashMap::new(),
                source: Default::default(),
            },
            atom_map: HashMap::new(),
        }
    }
    pub fn import(&mut self, fg: &FlowGraph) -> HashMap<NodeIndex, NodeIndex> {
        let mut mapping = HashMap::new();
        for node in fg.graph.node_indices() {
            let new_node = self.flow_graph.graph.add_node(fg.graph[node].clone());
            mapping.insert(node, new_node);
        }
        for edge in fg.graph.edge_indices() {
            let (source, target) = fg.graph.edge_endpoints(edge).unwrap();
            self.flow_graph.graph.add_edge(
                mapping[&source],
                mapping[&target],
                fg.graph[edge].clone(),
            );
        }
        self.flow_graph.source.extend(fg.source.sources.clone());
        mapping
    }
    pub fn add_input_port(&mut self, kind: Kind, index: usize) {
        for path in kind.all_leafs() {
            let node_kind = NodeKind::Buffer(BufferRef {
                name: format!("{} port{} input{:?}", self.flow_graph.name, index, path),
                kind,
                path: path.clone(),
            });
            let node_index = self.flow_graph.graph.add_node(node_kind.clone());
            self.flow_graph
                .ports
                .insert(NodePort::Input(index, path), node_index);
            self.atom_map.insert(node_kind, node_index);
        }
    }
    pub fn add_output_port(&mut self, kind: Kind) {
        for path in kind.all_leafs() {
            let node_kind = NodeKind::Buffer(BufferRef {
                name: format!("{} output{:?}", self.flow_graph.name, path),
                kind,
                path: path.clone(),
            });
            let node_index = self.flow_graph.graph.add_node(node_kind.clone());
            self.flow_graph
                .ports
                .insert(NodePort::Output(path), node_index);
            self.atom_map.insert(node_kind, node_index);
        }
    }
    pub fn forward_input_to_child(
        &mut self,
        input_kind: Kind,
        my_base_path: &Path,
        my_input_index: usize,
        child_flow_graph: &FlowGraph,
        child_input_index: usize,
        node_map: &HashMap<NodeIndex, NodeIndex>,
    ) -> Result<(), RHDLError> {
        for path in input_kind.all_leafs() {
            let my_port =
                self.get_input_port(my_input_index, &(my_base_path.clone()).join(&path))?;
            let child_port = child_flow_graph.input_port(child_input_index, &path)?;
            let child_node_in_my_graph = node_map[&child_port];
            self.add_edge(
                my_port,
                child_node_in_my_graph,
                EdgeKind::InputForwardToChild,
            );
        }
        Ok(())
    }
    pub fn forward_output_from_child(
        &mut self,
        output_kind: Kind,
        my_base_path: &Path,
        child_flow_graph: &FlowGraph,
        node_map: &HashMap<NodeIndex, NodeIndex>,
    ) -> Result<(), RHDLError> {
        for path in output_kind.all_leafs() {
            let my_port = self.get_output_port(&(my_base_path.clone()).join(&path))?;
            let child_port = child_flow_graph.output_port(&path)?;
            let child_node_in_my_graph = node_map[&child_port];
            self.add_edge(
                child_node_in_my_graph,
                my_port,
                EdgeKind::OutputForwardFromChild,
            );
        }
        Ok(())
    }
    pub fn get_input_port(&self, index: usize, path: &Path) -> Result<NodeIndex, RHDLError> {
        self.flow_graph.input_port(index, path)
    }
    pub fn get_output_port(&self, path: &Path) -> Result<NodeIndex, RHDLError> {
        self.flow_graph.output_port(path)
    }
    pub fn add_edge(&mut self, source: NodeIndex, target: NodeIndex, kind: EdgeKind) {
        self.flow_graph.graph.add_edge(source, target, kind);
    }
    pub fn add_constant(&mut self, name: &str, value: TypedBits, path: Path) -> NodeIndex {
        let node_kind = NodeKind::Constant(ConstantRef {
            name: name.to_string(),
            value,
            path,
        });
        if let Some(node_index) = self.atom_map.get(&node_kind) {
            *node_index
        } else {
            let node_index = self.flow_graph.graph.add_node(node_kind.clone());
            self.atom_map.insert(node_kind, node_index);
            node_index
        }
    }
    pub fn build(self) -> FlowGraph {
        self.flow_graph
    }
}
