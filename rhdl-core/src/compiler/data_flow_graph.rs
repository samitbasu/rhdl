use crate::rhif::rhif_spec::Array;
use crate::rhif::rhif_spec::Assign;
use crate::rhif::rhif_spec::Binary;
use crate::rhif::rhif_spec::Block;
use crate::rhif::rhif_spec::Cast;
use crate::rhif::rhif_spec::If;
use crate::rhif::rhif_spec::OpCode;
use crate::rhif::rhif_spec::Slot;
use crate::rhif::rhif_spec::Unary;
use crate::rhif::Object;
use crate::Design;
use anyhow::Result;
use petgraph::graph::NodeIndex;
use petgraph::Graph;
use std::collections::HashMap;

type DataFlowGraph = Graph<Slot, OpCode>;

struct DataFlowGraphContext<'a> {
    dfg: DataFlowGraph,
    slot_to_node: HashMap<Slot, NodeIndex>,
    blocks: &'a [Block],
    design: &'a Design,
    obj: &'a Object,
    early_return_encountered: bool,
}

impl<'a> DataFlowGraphContext<'a> {
    fn node(&mut self, slot: &Slot) -> Result<NodeIndex> {
        match self.slot_to_node.entry(*slot) {
            std::collections::hash_map::Entry::Occupied(entry) => Ok(*entry.get()),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let node = self.dfg.add_node(*slot);
                entry.insert(node);
                Ok(node)
            }
        }
    }

    fn op(&mut self, op: &OpCode) -> Result<()> {
        match op {
            OpCode::Binary(Binary {
                op: _,
                lhs,
                arg1,
                arg2,
            }) => {
                let lhs_node = self.node(lhs)?;
                let arg1_node = self.node(arg1)?;
                let arg2_node = self.node(arg2)?;
                self.dfg.add_edge(arg1_node, lhs_node, *op);
                self.dfg.add_edge(arg2_node, lhs_node, *op);
            }
            OpCode::Unary(Unary { op: _, lhs, arg1 }) => {
                let arg_node = self.node(arg1)?;
                let lhs_node = self.node(lhs)?;
                self.dfg.add_edge(arg_node, lhs_node, *op);
            }
            OpCode::If(If {
                lhs,
                cond,
                then_branch,
                else_branch,
            }) => {
                let cond_node = self.node(cond)?;
                let lhs_node = self.node(lhs)?;
                self.dfg.add_edge(cond_node, lhs_node, *op);
                self.block(then_branch)?;
                self.block(else_branch)?;
            }
            OpCode::Array(Array { lhs, elements }) => {
                let lhs_node = self.node(lhs)?;
                for element in elements {
                    let element_node = self.node(element)?;
                    self.dfg.add_edge(element_node, lhs_node, *op);
                }
            }
            OpCode::AsBits(Cast { lhs, arg, len }) => {
                let arg_node = self.node(arg)?;
                let lhs_node = self.node(lhs)?;
                self.dfg.add_edge(arg_node, lhs_node, *op);
            }
            OpCode::AsSigned(Cast { lhs, arg, len }) => {
                let arg_node = self.node(arg)?;
                let lhs_node = self.node(lhs)?;
                self.dfg.add_edge(arg_node, lhs_node, *op);
            }
            OpCode::Assign(Assign { lhs, rhs, path }) => {
                let rhs_node = self.node(rhs)?;
                let lhs_node = self.node(lhs)?;
                self.dfg.add_edge(rhs_node, lhs_node, *op);
                for slot in path.dynamic_slots() {
                    let slot_node = self.node(&slot)?;
                    self.dfg.add_edge(slot_node, lhs_node, *op);
                }
            }
        }
        Ok(())
    }
}
