use crate::rhif::spec::{
    Array, Assign, Binary, Case, Cast, Discriminant, Enum, Exec, Index, OpCode, Repeat, Select,
    Slot, Splice, Struct, Tuple, Unary,
};
use crate::rhif::Object;
use anyhow::Result;
use petgraph::graph::NodeIndex;
use petgraph::Graph;
use std::collections::HashMap;

type DataFlowGraph = Graph<Slot, OpCode>;

struct DataFlowGraphContext {
    dfg: DataFlowGraph,
    slot_to_node: HashMap<Slot, NodeIndex>,
}

pub fn make_data_flow(object: &Object) -> Result<DataFlowGraph> {
    let mut ctx = DataFlowGraphContext {
        dfg: DataFlowGraph::new(),
        slot_to_node: HashMap::new(),
    };
    ctx.ops(&object.ops)?;
    Ok(ctx.dfg)
}

impl DataFlowGraphContext {
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
    fn ops(&mut self, ops: &[OpCode]) -> Result<()> {
        for op in ops {
            self.op(op)?;
        }
        Ok(())
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
                self.dfg.add_edge(arg1_node, lhs_node, op.clone());
                self.dfg.add_edge(arg2_node, lhs_node, op.clone());
            }
            OpCode::Unary(Unary { op: _, lhs, arg1 }) => {
                let arg_node = self.node(arg1)?;
                let lhs_node = self.node(lhs)?;
                self.dfg.add_edge(arg_node, lhs_node, op.clone());
            }
            OpCode::Select(Select {
                lhs,
                cond,
                true_value,
                false_value,
            }) => {
                let cond_node = self.node(cond)?;
                let true_value_node = self.node(true_value)?;
                let false_value_node = self.node(false_value)?;
                let lhs_node = self.node(lhs)?;
                self.dfg.add_edge(cond_node, lhs_node, op.clone());
                self.dfg.add_edge(true_value_node, lhs_node, op.clone());
                self.dfg.add_edge(false_value_node, lhs_node, op.clone());
            }
            OpCode::Array(Array { lhs, elements }) => {
                let lhs_node = self.node(lhs)?;
                for element in elements {
                    let element_node = self.node(element)?;
                    self.dfg.add_edge(element_node, lhs_node, op.clone());
                }
            }
            OpCode::AsBits(Cast { lhs, arg, len }) => {
                let arg_node = self.node(arg)?;
                let lhs_node = self.node(lhs)?;
                self.dfg.add_edge(arg_node, lhs_node, op.clone());
            }
            OpCode::AsSigned(Cast { lhs, arg, len }) => {
                let arg_node = self.node(arg)?;
                let lhs_node = self.node(lhs)?;
                self.dfg.add_edge(arg_node, lhs_node, op.clone());
            }
            OpCode::Assign(Assign { lhs, rhs }) => {
                let rhs_node = self.node(rhs)?;
                let lhs_node = self.node(lhs)?;
                self.dfg.add_edge(rhs_node, lhs_node, op.clone());
            }
            OpCode::Splice(Splice {
                lhs,
                orig,
                path,
                subst,
            }) => {
                let orig_node = self.node(orig)?;
                let lhs_node = self.node(lhs)?;
                let subst_node = self.node(subst)?;
                self.dfg.add_edge(orig_node, lhs_node, op.clone());
                for slot in path.dynamic_slots() {
                    let slot_node = self.node(&slot)?;
                    self.dfg.add_edge(slot_node, lhs_node, op.clone());
                }
                self.dfg.add_edge(subst_node, lhs_node, op.clone());
            }
            OpCode::Index(Index { lhs, arg, path }) => {
                let arg_node = self.node(arg)?;
                let lhs_node = self.node(lhs)?;
                self.dfg.add_edge(arg_node, lhs_node, op.clone());
                for slot in path.dynamic_slots() {
                    let slot_node = self.node(slot)?;
                    self.dfg.add_edge(slot_node, lhs_node, op.clone());
                }
            }
            OpCode::Repeat(Repeat { lhs, value, len }) => {
                let value_node = self.node(value)?;
                let lhs_node = self.node(lhs)?;
                self.dfg.add_edge(value_node, lhs_node, op.clone());
            }
            OpCode::Struct(Struct {
                lhs,
                fields,
                rest,
                template,
            }) => {
                let lhs_node = self.node(lhs)?;
                for field in fields {
                    let field_node = self.node(&field.value)?;
                    self.dfg.add_edge(field_node, lhs_node, op.clone());
                }
                if let Some(rest) = rest {
                    let rest_node = self.node(rest)?;
                    self.dfg.add_edge(rest_node, lhs_node, op.clone());
                }
            }
            OpCode::Tuple(Tuple { lhs, fields }) => {
                let lhs_node = self.node(lhs)?;
                for field in fields {
                    let field_node = self.node(&field)?;
                    self.dfg.add_edge(field_node, lhs_node, op.clone());
                }
            }
            OpCode::Case(Case {
                lhs,
                discriminant,
                table,
            }) => {
                let discriminant_node = self.node(discriminant)?;
                let lhs_node = self.node(lhs)?;
                self.dfg.add_edge(discriminant_node, lhs_node, op.clone());
                for (value, slot) in table {
                    let slot_node = self.node(slot)?;
                    self.dfg.add_edge(slot_node, lhs_node, op.clone());
                }
            }
            OpCode::Exec(Exec { lhs, id, args }) => {
                let lhs_node = self.node(lhs)?;
                for arg in args {
                    let arg_node = self.node(arg)?;
                    self.dfg.add_edge(arg_node, lhs_node, op.clone());
                }
            }
            OpCode::Discriminant(Discriminant { lhs, arg }) => {
                let arg_node = self.node(arg)?;
                let lhs_node = self.node(lhs)?;
                self.dfg.add_edge(arg_node, lhs_node, op.clone());
            }
            OpCode::Enum(Enum {
                lhs,
                fields,
                template,
            }) => {
                let lhs_node = self.node(lhs)?;
                for field in fields {
                    let field_node = self.node(&field.value)?;
                    self.dfg.add_edge(field_node, lhs_node, op.clone());
                }
            }
            OpCode::Comment(_) => {}
        }
        Ok(())
    }
}
