use crate::rhif::spec::{
    Array, Assign, Binary, Case, Cast, Discriminant, Enum, Exec, Index, OpCode, Repeat, Select,
    Slot, Splice, Struct, Tuple, Unary,
};
use crate::rhif::Object;
use crate::{Design, KernelFnKind};
use anyhow::anyhow;
use anyhow::{bail, Result};
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::Graph;
use std::collections::HashMap;

type DataFlowGraphType = Graph<Slot, OpCode>;

pub struct DataFlowGraph(DataFlowGraphType);

#[derive(Debug, Clone, PartialEq, Default, Copy)]
struct Relocation {
    register_offset: usize,
    literal_offset: usize,
}

impl Relocation {
    fn relocate(&self, slot: &Slot) -> Slot {
        match slot {
            Slot::Register(ndx) => Slot::Register(ndx + self.register_offset),
            Slot::Literal(ndx) => Slot::Literal(ndx + self.literal_offset),
            Slot::Empty => Slot::Empty,
        }
    }
}

struct DataFlowGraphContext<'a> {
    dfg: DataFlowGraphType,
    slot_to_node: HashMap<Slot, NodeIndex>,
    next_free: Relocation,
    base: Relocation,
    object: &'a Object,
    design: &'a Design,
}

pub fn make_data_flow(design: &Design) -> Result<DataFlowGraph> {
    let top = &design.objects[&design.top];
    let mut ctx = DataFlowGraphContext {
        dfg: Default::default(),
        slot_to_node: HashMap::new(),
        next_free: Default::default(),
        base: Default::default(),
        object: top,
        design,
    };
    ctx.base = ctx.allocate(top);
    ctx.func()?;
    Ok(DataFlowGraph(ctx.dfg))
}

impl DataFlowGraph {
    pub fn dot(&self) -> String {
        format!("{}", Dot::with_config(&self.0, Default::default()))
    }
}

impl<'a> DataFlowGraphContext<'a> {
    fn allocate(&mut self, obj: &Object) -> Relocation {
        let result = self.next_free.clone();
        self.next_free.register_offset += obj.reg_max_index() + 1;
        self.next_free.literal_offset += obj.literal_max_index() + 1;
        result
    }
    fn node(&mut self, slot: &Slot) -> Result<NodeIndex> {
        let slot = self.base.relocate(slot);
        match self.slot_to_node.entry(slot) {
            std::collections::hash_map::Entry::Occupied(entry) => Ok(*entry.get()),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let node = self.dfg.add_node(slot);
                entry.insert(node);
                Ok(node)
            }
        }
    }
    fn func(&mut self) -> Result<()> {
        for op in &self.object.ops {
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
                if !lhs.is_empty() {
                    let cond_node = self.node(cond)?;
                    let true_value_node = self.node(true_value)?;
                    let false_value_node = self.node(false_value)?;
                    let lhs_node = self.node(lhs)?;
                    self.dfg.add_edge(cond_node, lhs_node, op.clone());
                    self.dfg.add_edge(true_value_node, lhs_node, op.clone());
                    self.dfg.add_edge(false_value_node, lhs_node, op.clone());
                }
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
                // Inline the called function.  To do this, we need to first
                // calculate the register offset for the called function.
                // We do this by taking the current offset and adding enough
                // registers and literals to account for our needs.

                // Get the register names in our current scope
                let lhs_in_my_scope = self.node(lhs)?;
                let args_in_my_scope = args
                    .iter()
                    .map(|arg| self.node(arg).map(|n| (n, *arg)))
                    .collect::<Result<Vec<_>>>()?;

                let func = &self.object.externals[id.0];
                let KernelFnKind::Kernel(kernel) = &func.code else {
                    bail!("DFG does not currently support external function defs")
                };
                let callee = self
                    .design
                    .objects
                    .get(&kernel.fn_id)
                    .ok_or(anyhow!("ICE Could not find function referenced in design"))?;
                let callee_base = self.allocate(callee);

                // save our base
                let base = self.base;
                let object = self.object;
                self.base = callee_base;
                self.object = callee;

                // Link the arguments as reading from our scope and importing into the function scope
                for (arg, arg_in_my_scope) in callee.arguments.iter().zip(args_in_my_scope) {
                    let arg_in_callee_scope = self.node(arg)?;
                    self.dfg.add_edge(
                        arg_in_my_scope.0,
                        arg_in_callee_scope,
                        OpCode::Assign(Assign {
                            lhs: arg_in_my_scope.1,
                            rhs: *arg,
                        }),
                    );
                }

                // Link the return value as reading from the function scope and importing into our scope
                let lhs_in_callee_scope = self.node(&callee.return_slot)?;
                self.dfg.add_edge(
                    lhs_in_callee_scope,
                    lhs_in_my_scope,
                    OpCode::Assign(Assign {
                        lhs: callee.return_slot,
                        rhs: callee.return_slot,
                    }),
                );

                self.func()?;
                self.base = base;
                self.object = object;
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
