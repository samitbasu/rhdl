// We want to estimate the cost of computing some
// element of the output of the given function.

use std::collections::HashMap;

use petgraph::{graph::NodeIndex, Graph};

use crate::{
    ast::ast_impl::FunctionId,
    rhif::{
        spec::{CaseArgument, OpCode, Slot},
        Object,
    },
    types::path::{bit_range, path_star, Path},
    Module, RHDLError,
};

use super::{slot_to_opcode::slot_to_opcode, utils::path_with_member};

pub trait CostEstimator {
    fn cost(&self, obj: &Object, opcode: usize) -> f64;
}

pub struct TimingAnalysis<'a> {
    module: &'a Module,
    function: FunctionId,
    object: &'a Object,
    slot_to_opcode: HashMap<Slot, usize>,
    computer: &'a dyn CostEstimator,
}

pub struct CostTrace {
    pub trace: Vec<Slot>,
    pub cost: f64,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Node {
    slot: Slot,
    path: Path,
    function: FunctionId,
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} ({:?}) [{:?}]", self.slot, self.function, self.path)
    }
}

#[derive(Default, Debug)]
pub struct CostGraph {
    pub graph: Graph<Node, f64>,
    pub map: HashMap<Node, NodeIndex>,
    pub output: NodeIndex,
    pub arguments: Vec<NodeIndex>,
}

impl CostGraph {
    fn add_edge(&mut self, to: NodeIndex, from: NodeIndex, cost: f64) {
        self.graph.add_edge(from, to, cost);
    }
    fn merge(&mut self, other: &CostGraph) -> HashMap<NodeIndex, NodeIndex> {
        let mut cross_map = HashMap::new();
        for nix in other.graph.node_indices() {
            let node = other.graph[nix].clone();
            let ndx = self.graph.add_node(node.clone());
            self.map.insert(node, ndx);
            cross_map.insert(nix, ndx);
        }
        for edge in other.graph.edge_indices() {
            let (from, to) = other.graph.edge_endpoints(edge).unwrap();
            let from = cross_map[&from];
            let to = cross_map[&to];
            let cost = other.graph[edge];
            self.add_edge(to, from, cost);
        }
        cross_map
    }
    fn inputs(&self) -> impl Iterator<Item = (Slot, Path)> + '_ {
        self.arguments.iter().map(move |ndx| {
            let node = &self.graph[*ndx];
            (node.slot, node.path.clone())
        })
    }
}

pub fn compute_timing_graph(
    module: &Module,
    function: FunctionId,
    path: &Path,
    computer: &dyn CostEstimator,
) -> Result<CostGraph, RHDLError> {
    let mut analysis = TimingAnalysis::new(module, function, computer);
    let mut cost_graph = Default::default();
    let return_index = analysis.cost(analysis.object.return_slot, path, &mut cost_graph)?;
    cost_graph.output = return_index;
    // Collect all nodes that are input arguments of the function
    cost_graph.arguments = cost_graph
        .map
        .iter()
        .filter_map(|(node, ndx)| {
            if analysis.object.arguments.contains(&node.slot) {
                Some(*ndx)
            } else {
                None
            }
        })
        .collect();
    /*

       let opt_path = bellman_ford(&cost_graph.graph, return_index).unwrap();
       let root_path = &analysis.paths[&path];
       eprintln!("path: {:?}", path);
       eprintln!("argument_node_indices: {:?}", argument_node_indices);
       for ndx in argument_node_indices {
           let argument_slot = cost_graph.graph[ndx].slot;
           let argument_path = &analysis.paths[&cost_graph.graph[ndx].path];
           let cost_to_argument = opt_path.distances[ndx.index()];
           eprintln!(
               "Cost from {:?}[{:?}] to {:?}[{:?}] is {}",
               analysis.object.return_slot, root_path, argument_slot, argument_path, cost_to_argument
           );
       }
    */
    Ok(cost_graph)
}

impl<'a> TimingAnalysis<'a> {
    pub fn new(module: &'a Module, function: FunctionId, computer: &'a dyn CostEstimator) -> Self {
        let object = &module.objects[&function];
        let slot_to_opcode = slot_to_opcode(object);
        Self {
            module,
            object,
            function,
            slot_to_opcode,
            computer,
        }
    }

    // TODO - this is recursive.  It should probably be rewritten without recursion.
    fn cost(
        &mut self,
        slot: Slot,
        output_path: &Path,
        cost_graph: &mut CostGraph,
    ) -> Result<NodeIndex, RHDLError> {
        let node = Node {
            slot,
            path: output_path.clone(),
            function: self.function,
        };
        if let Some(ndx) = cost_graph.map.get(&node) {
            return Ok(*ndx);
        }
        let ndx = cost_graph.graph.add_node(node.clone());
        cost_graph.map.insert(node, ndx);
        if self.object.arguments.contains(&slot) {
            return Ok(ndx);
        }
        if !self.slot_to_opcode.contains_key(&slot) {
            // This is a literal
            return Ok(ndx);
        }
        eprintln!("Compute timing cost for slot {:?}", slot);
        let opcode = self.slot_to_opcode[&slot];
        let cost = self.computer.cost(self.object, opcode);
        let opcode = &self.object.ops[opcode];
        match opcode {
            OpCode::Array(array) => {
                if let Some(upstream) = array
                    .elements
                    .iter()
                    .enumerate()
                    .find(|(ndx, _ix)| Path::default().index(*ndx).is_prefix_of(output_path))
                {
                    let path = output_path
                        .clone()
                        .strip_prefix(&Path::default().index(upstream.0))?;
                    let arg = self.cost(*upstream.1, &path, cost_graph)?;
                    cost_graph.add_edge(arg, ndx, cost);
                } else {
                    // We need the whole array, so we need to compute the whole array
                    for child in &array.elements {
                        let arg = self.cost(*child, output_path, cost_graph)?;
                        cost_graph.add_edge(arg, ndx, cost);
                    }
                }
            }
            OpCode::AsBits(cast) | OpCode::AsSigned(cast) => {
                let arg = self.cost(cast.arg, output_path, cost_graph)?;
                cost_graph.add_edge(arg, ndx, cost);
            }
            OpCode::Assign(assign) => {
                let arg = self.cost(assign.rhs, output_path, cost_graph)?;
                cost_graph.add_edge(arg, ndx, cost);
            }
            OpCode::Binary(binary) => {
                let arg1 = self.cost(binary.arg1, output_path, cost_graph)?;
                let arg2 = self.cost(binary.arg2, output_path, cost_graph)?;
                cost_graph.add_edge(arg1, ndx, cost);
                cost_graph.add_edge(arg2, ndx, cost);
            }
            OpCode::Case(case) => {
                let discriminant = self.cost(case.discriminant, &Default::default(), cost_graph)?;
                cost_graph.add_edge(discriminant, ndx, cost);
                for child in &case.table {
                    let arg = self.cost(child.1, output_path, cost_graph)?;
                    cost_graph.add_edge(arg, ndx, cost);
                    if let CaseArgument::Slot(slot) = child.0 {
                        let arg = self.cost(slot, output_path, cost_graph)?;
                        cost_graph.add_edge(arg, ndx, cost);
                    }
                }
            }
            OpCode::Comment(_) => {}
            OpCode::Enum(e) => {
                let discriminant = e.template.discriminant()?.as_i64()?;
                if let Some(field) = e.fields.iter().find(|field| {
                    path_with_member(
                        Path::default().payload_by_value(discriminant),
                        &field.member,
                    )
                    .is_prefix_of(output_path)
                }) {
                    let path = output_path.clone().strip_prefix(&path_with_member(
                        Path::default().payload_by_value(discriminant),
                        &field.member,
                    ))?;
                    let arg = self.cost(field.value, &path, cost_graph)?;
                    cost_graph.add_edge(arg, ndx, cost);
                } else {
                    for field in &e.fields {
                        let arg = self.cost(field.value, &output_path, cost_graph)?;
                        cost_graph.add_edge(arg, ndx, cost);
                    }
                }
            }
            OpCode::Exec(exec) => {
                let code = self.object.externals[&exec.id].code.inner();
                let exec_id = code.fn_id;
                let kernel_rhif = &self.module.objects[&exec_id];
                let kernel_graph =
                    compute_timing_graph(self.module, exec_id, output_path, self.computer)?;
                // Merge the kernel graph into the cost graph
                let cross_map = cost_graph.merge(&kernel_graph);
                cost_graph.add_edge(cross_map[&kernel_graph.output], ndx, cost);
                // Handle the arguments...
                for nix in kernel_graph.arguments {
                    let node = kernel_graph.graph[nix].clone();
                    // Figure out which argument this is
                    let argument_index = kernel_rhif
                        .arguments
                        .iter()
                        .position(|&arg| arg == node.slot)
                        .unwrap();
                    // This is now mapped to exec's list of slots
                    let arg = self.cost(exec.args[argument_index], &node.path, cost_graph)?;
                    cost_graph.add_edge(arg, cross_map[&nix], cost);
                }
            }
            OpCode::Index(index) => {
                for dynamic in index.path.dynamic_slots() {
                    let arg = self.cost(*dynamic, &Default::default(), cost_graph)?;
                    cost_graph.add_edge(arg, ndx, cost);
                }
                let kind = &self.object.kind[&index.arg];
                for path in path_star(kind, &index.path)? {
                    let arg = self.cost(index.arg, &path, cost_graph)?;
                    cost_graph.add_edge(arg, ndx, cost);
                }
            }
            OpCode::Noop => {}
            OpCode::Repeat(repeat) => {
                let mut covered = false;
                for i in 0..repeat.len {
                    if Path::default().index(i as _).is_prefix_of(output_path) {
                        let path = output_path
                            .clone()
                            .strip_prefix(&Path::default().index(i as _))?;
                        let arg = self.cost(repeat.value, &path, cost_graph)?;
                        cost_graph.add_edge(arg, ndx, cost);
                        covered = true;
                    }
                }
                if !covered {
                    let arg = self.cost(repeat.value, output_path, cost_graph)?;
                    cost_graph.add_edge(arg, ndx, cost);
                }
            }
            OpCode::Retime(retime) => {
                let arg = self.cost(retime.arg, output_path, cost_graph)?;
                cost_graph.add_edge(arg, ndx, cost);
            }
            OpCode::Select(select) => {
                let arg1 = self.cost(select.true_value, output_path, cost_graph)?;
                let arg2 = self.cost(select.false_value, output_path, cost_graph)?;
                cost_graph.add_edge(arg1, ndx, cost);
                cost_graph.add_edge(arg2, ndx, cost);
                let cond = self.cost(select.cond, &Default::default(), cost_graph)?;
                cost_graph.add_edge(cond, ndx, cost);
            }
            OpCode::Splice(splice) => {
                for dynamic in splice.path.dynamic_slots() {
                    let arg = self.cost(*dynamic, &Default::default(), cost_graph)?;
                    cost_graph.add_edge(arg, ndx, cost);
                }
                let kind = &self.object.kind[&splice.orig];
                let (output_bit_range, _) = bit_range(kind.clone(), output_path)?;
                let mut any_upstream = false;
                for s_path in path_star(kind, &splice.path)? {
                    let (replace_bit_range, _) = bit_range(kind.clone(), &s_path)?;
                    let output_path_in_replacement =
                        replace_bit_range.contains(&output_bit_range.start);
                    if output_path_in_replacement {
                        let path = output_path.clone().strip_prefix(&s_path)?;
                        let arg = self.cost(splice.subst, &path, cost_graph)?;
                        cost_graph.add_edge(arg, ndx, cost);
                        any_upstream = true;
                    }
                }
                if !any_upstream {
                    let arg = self.cost(splice.orig, output_path, cost_graph)?;
                    cost_graph.add_edge(arg, ndx, cost);
                }
            }
            OpCode::Struct(strukt) => {
                if let Some(field) = strukt.fields.iter().find(|field| {
                    path_with_member(Path::default(), &field.member).is_prefix_of(output_path)
                }) {
                    let path = output_path
                        .clone()
                        .strip_prefix(&path_with_member(Path::default(), &field.member))?;
                    let arg = self.cost(field.value, &path, cost_graph)?;
                    cost_graph.add_edge(arg, ndx, cost);
                } else if let Some(rest) = strukt.rest {
                    let arg = self.cost(rest, output_path, cost_graph)?;
                    cost_graph.add_edge(arg, ndx, cost);
                } else {
                    for field in &strukt.fields {
                        let arg = self.cost(field.value, output_path, cost_graph)?;
                        cost_graph.add_edge(arg, ndx, cost);
                    }
                    if let Some(rest) = strukt.rest {
                        let arg = self.cost(rest, output_path, cost_graph)?;
                        cost_graph.add_edge(arg, ndx, cost);
                    }
                }
            }
            OpCode::Tuple(tuple) => {
                let mut covered = false;
                for (i, child) in tuple.fields.iter().enumerate() {
                    if Path::default()
                        .tuple_index(i as _)
                        .is_prefix_of(output_path)
                    {
                        let path = output_path
                            .clone()
                            .strip_prefix(&Path::default().tuple_index(i as _))?;
                        let arg = self.cost(*child, &path, cost_graph)?;
                        cost_graph.add_edge(arg, ndx, cost);
                        covered = true;
                    }
                }
                if !covered {
                    for child in &tuple.fields {
                        let arg = self.cost(*child, output_path, cost_graph)?;
                        cost_graph.add_edge(arg, ndx, cost);
                    }
                }
            }
            OpCode::Unary(unary) => {
                let arg = self.cost(unary.arg1, output_path, cost_graph)?;
                cost_graph.add_edge(arg, ndx, cost);
            }
        }
        Ok(ndx)
    }
}
