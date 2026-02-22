use std::collections::HashMap;

use petgraph::graph::NodeIndex;

use crate::{
    Kind, RHDLError, TypedBits,
    flow_graph::{BufferRef, ConstantRef, EdgeKind, FlowGraph, NodeKind, NodePort, PortSet},
    types::path::{Path, PathExt, sub_kind},
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
                ports: Default::default(),
                source: Default::default(),
            },
            atom_map: HashMap::new(),
        }
    }
    pub fn import(&mut self, fg: FlowGraph) -> PortSet {
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
        fg.ports
            .into_iter()
            .map(|(port, node)| (port.clone(), mapping[&node]))
            .collect()
    }
    pub fn add_input_port(&mut self, kind: Kind, index: usize) -> Result<(), RHDLError> {
        for path in kind.all_leafs() {
            let leaf_kind = sub_kind(kind, &path)?;
            let node_kind = NodeKind::Buffer(BufferRef {
                name: format!("{} port{} input{:?}", self.flow_graph.name, index, path),
                kind,
                path: path.clone(),
                leaf_kind,
            });
            let node_index = self.flow_graph.graph.add_node(node_kind.clone());
            self.flow_graph
                .ports
                .insert(NodePort::Input(index, path), node_index);
            self.atom_map.insert(node_kind, node_index);
        }
        Ok(())
    }
    pub fn add_output_port(&mut self, kind: Kind) -> Result<(), RHDLError> {
        for path in kind.all_leafs() {
            let leaf_kind = sub_kind(kind, &path)?;
            let node_kind = NodeKind::Buffer(BufferRef {
                name: format!("{} output{:?}", self.flow_graph.name, path),
                kind,
                path: path.clone(),
                leaf_kind,
            });
            let node_index = self.flow_graph.graph.add_node(node_kind.clone());
            self.flow_graph
                .ports
                .insert(NodePort::Output(path), node_index);
            self.atom_map.insert(node_kind, node_index);
        }
        Ok(())
    }
    pub fn forward_input_to_child(
        &mut self,
        input_kind: Kind,
        my_base_path: &Path,
        my_input_index: usize,
        child_ports: &PortSet,
        child_input_index: usize,
    ) -> Result<(), RHDLError> {
        for path in input_kind.all_leafs() {
            let my_port =
                self.get_input_port(my_input_index, &(my_base_path.clone()).join(&path))?;
            let child_port = child_ports.input_port(child_input_index, &path)?;
            self.add_edge(my_port, child_port, EdgeKind::InputForwardToChild)?;
        }
        Ok(())
    }
    pub fn forward_output_from_child(
        &mut self,
        output_kind: Kind,
        my_base_path: &Path,
        child_ports: &PortSet,
    ) -> Result<(), RHDLError> {
        for path in output_kind.all_leafs() {
            let my_port = self.get_output_port(&(my_base_path.clone()).join(&path))?;
            let child_port = child_ports.output_port(&path)?;
            self.add_edge(child_port, my_port, EdgeKind::OutputForwardFromChild)?;
        }
        Ok(())
    }
    pub fn get_input_port(&self, index: usize, path: &Path) -> Result<NodeIndex, RHDLError> {
        self.flow_graph.input_port(index, path)
    }
    pub fn get_output_port(&self, path: &Path) -> Result<NodeIndex, RHDLError> {
        self.flow_graph.output_port(path)
    }
    pub fn add_edge(
        &mut self,
        source: NodeIndex,
        target: NodeIndex,
        kind: EdgeKind,
    ) -> Result<(), RHDLError> {
        let source_leaf = self.flow_graph.graph[source].leaf_kind();
        let target_leaf = self.flow_graph.graph[target].leaf_kind();
        if source_leaf != target_leaf {
            panic!(
                "Mismatched leaf kinds in edge from {:?} to {:?}: {:?} vs {:?}",
                source, target, source_leaf, target_leaf
            );
        }
        self.flow_graph.graph.add_edge(source, target, kind);
        Ok(())
    }
    pub fn add_constant(
        &mut self,
        name: &str,
        value: TypedBits,
        path: Path,
    ) -> Result<NodeIndex, RHDLError> {
        let leaf_kind = sub_kind(value.kind(), &path)?;
        let node_kind = NodeKind::Constant(ConstantRef {
            name: name.to_string(),
            value,
            path,
            leaf_kind,
        });
        Ok(if let Some(node_index) = self.atom_map.get(&node_kind) {
            *node_index
        } else {
            let node_index = self.flow_graph.graph.add_node(node_kind.clone());
            self.atom_map.insert(node_kind, node_index);
            node_index
        })
    }
    pub fn build(self) -> FlowGraph {
        self.flow_graph
    }
}
