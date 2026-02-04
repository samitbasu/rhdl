use std::{collections::HashMap, sync::Arc};

use internment::Intern;
use petgraph::graph::NodeIndex;

use crate::{
    common::symtab::RegisterId,
    rhif::{
        Object,
        object::LocatedOpCode,
        spec::{Slot, SlotKind},
    },
    types::atom::{AtomPath, iter_atoms},
};

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct AtomRef {
    pub object: Intern<Object>,
    pub slot: Slot,
    pub atom: AtomPath,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum EdgeKind {
    OpCode(LocatedOpCode),
}

#[derive(Clone)]
pub struct FlowGraph {
    pub heads: Vec<NodeIndex>,
    pub graph: petgraph::graph::DiGraph<AtomRef, EdgeKind>,
    pub tails: Vec<NodeIndex>,
}

pub(crate) fn build_flow_graph(object: Intern<Object>) -> FlowGraph {
    // Import the arguments
    let mut graph = petgraph::graph::DiGraph::<AtomRef, EdgeKind>::new();
    let mut heads = Vec::new();
    let mut tails = Vec::new();
    for arg in object.arguments.iter() {
        for atom in iter_atoms(object.symtab[*arg]) {
            let atom = AtomRef {
                object,
                slot: arg.into(),
                atom,
            };
            let node = graph.add_node(atom);
            heads.push(node);
        }
    }
    for atom in iter_atoms(object.kind(object.return_slot)) {
        let atom = AtomRef {
            object,
            slot: object.return_slot,
            atom,
        };
        let node = graph.add_node(atom);
        tails.push(node);
    }

    todo!()
}
