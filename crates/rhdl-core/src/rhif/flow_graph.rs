use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use crate::{
    ast::{SourceLocation, SourcePool, ast_impl::WrapOp},
    error::rhdl_error,
    types::path::{PathElement, PathError, PathExt},
};
use internment::Intern;
use log::debug;
use miette::{Diagnostic, SourceSpan};
use petgraph::graph::NodeIndex;
use thiserror::Error;

use crate::{
    Kind, RHDLError,
    rhif::{
        Object,
        object::LocatedOpCode,
        spec::{OpCode, Slot},
    },
    types::path::{Path, sub_kind},
};

#[derive(Error, Debug, Diagnostic)]
pub enum FlowGraphErrorKind {
    #[error("Mismatched kinds in assignment: from {from:?} to {to:?}")]
    MismatchedKindsInAssignment { from: Kind, to: Kind },
    #[error("Cannot copy atoms from {from:?} to {to:?}[{to_base_path:?}]")]
    CannotCopyAtomsToPath {
        from: Kind,
        to: Kind,
        to_base_path: Path,
    },
    #[error(
        "Cannot copy atoms from {from:?}[{from_base_path:?}] ({sub_kind:?}) to {to:?}[{to_base_path:?}] ({to_sub_kind:?})"
    )]
    CannotCopyAtomsFromPathToPath {
        from: Kind,
        to: Kind,
        from_base_path: Path,
        to_base_path: Path,
        sub_kind: Kind,
        to_sub_kind: Kind,
    },
    #[error(
        "Mismatched kinds in arguments to external function: expected {expected:?} got {actual:?}"
    )]
    MismatchedKindsInExternalFunctionCall {
        expected: Vec<Kind>,
        actual: Vec<Kind>,
    },
    #[error("Node is missing in flow graph: {slot:?} with path {path:?}")]
    MissingNode { slot: Slot, path: Path },
    #[error(
        "Mismatched return kinds in external function call: expected {expected:?} got {actual:?}"
    )]
    MismatchedReturnKindsInExternalFunctionCall { expected: Kind, actual: Kind },
}

#[derive(Debug, Error)]
#[error("Flow Graph Error")]
pub struct FlowGraphError {
    pub kind: FlowGraphErrorKind,
    pub src: SourcePool,
    pub err_span: SourceSpan,
}

impl Diagnostic for FlowGraphError {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src)
    }
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.kind.help()
    }
    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        Some(Box::new(std::iter::once(
            miette::LabeledSpan::new_primary_with_span(Some(self.kind.to_string()), self.err_span),
        )))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PathRef {
    pub object: Intern<Object>,
    pub slot: Slot,
    pub path: Path,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum EdgeKind {
    OpCode(LocatedOpCode),
}

#[derive(Clone, Default)]
pub struct FlowGraph {
    pub graph: petgraph::graph::DiGraph<PathRef, EdgeKind>,
    pub atom_map: HashMap<PathRef, NodeIndex>,
}

// Enumerate all possible concrete paths that a dynamic path could take.
fn path_star(kind: Kind, path: &Path) -> Result<Vec<Path>, RHDLError> {
    debug!("path star called with kind {kind:?} and path {path:?}");
    if !path.any_dynamic() {
        return Ok(vec![path.clone()]);
    }
    if let Some(element) = path.iter().next() {
        match element {
            PathElement::DynamicIndex(_) => {
                let Kind::Array(array) = kind else {
                    return Err(rhdl_error(PathError::DynamicIndexOnNonArray {
                        element: *element,
                        kind,
                    }));
                };
                let mut paths = Vec::new();
                for i in 0..array.size {
                    let path = std::iter::once(PathElement::Index(i))
                        .chain(path.iter().copied().skip(1))
                        .collect::<Path>();
                    let child_paths = path_star(kind, &path)?;
                    paths.extend(child_paths);
                }
                return Ok(paths);
            }
            p => {
                // We have a non-dynamic path element, like a.foo
                // We want to apply it to get the subtype
                let prefix_path = Path::with_element(*p);
                // The resulting kind we compute with [sub_kind]
                let prefix_kind = sub_kind(kind, &prefix_path)?;
                // Get a residual path
                let suffix_path = path.strip_prefix(&prefix_path)?;
                // Recurse
                let mut suffix_star = path_star(prefix_kind, &suffix_path)?;
                suffix_star
                    .iter_mut()
                    .for_each(|item| *item = prefix_path.clone().join(item));
                return Ok(suffix_star);
            }
        }
    }
    Ok(vec![path.clone()])
}

impl FlowGraph {
    fn raise_error(
        &self,
        kind: FlowGraphErrorKind,
        object: Intern<Object>,
        loc: SourceLocation,
    ) -> RHDLError {
        Box::new(FlowGraphError {
            kind,
            src: object.symbols.source(),
            err_span: object.symbols.span(loc).into(),
        })
        .into()
    }
    fn lookup(
        &self,
        object: Intern<Object>,
        slot: Slot,
        path: Path,
        loc: SourceLocation,
    ) -> Result<NodeIndex, RHDLError> {
        let atom = PathRef {
            object,
            slot,
            path: path.clone(),
        };
        if let Some(&ndx) = self.atom_map.get(&atom) {
            Ok(ndx)
        } else {
            Err(self.raise_error(FlowGraphErrorKind::MissingNode { slot, path }, object, loc))
        }
    }
}

struct FlowGraphBuilder {
    fg: FlowGraph,
    object: Intern<Object>,
}

impl FlowGraphBuilder {
    fn new(object: Intern<Object>) -> Self {
        Self {
            fg: FlowGraph::default(),
            object,
        }
    }
    fn raise_error(&self, kind: FlowGraphErrorKind, loc: SourceLocation) -> RHDLError {
        Box::new(FlowGraphError {
            kind,
            src: self.object.symbols.source(),
            err_span: self.object.symbols.span(loc).into(),
        })
        .into()
    }
    fn import(&mut self, fg: &FlowGraph) -> HashMap<NodeIndex, NodeIndex> {
        let mut remap = HashMap::new();
        for ndx in fg.graph.node_indices() {
            let node = fg.graph[ndx].clone();
            let new_ndx = self.node(node);
            remap.insert(ndx, new_ndx);
        }
        for edge in fg.graph.edge_indices() {
            let edge_kind = &fg.graph[edge];
            let (start, end) = fg.graph.edge_endpoints(edge).unwrap();
            let new_start = remap[&start];
            let new_end = remap[&end];
            self.fg
                .graph
                .add_edge(new_start, new_end, edge_kind.clone());
        }
        remap
    }
    fn node(&mut self, path_ref: PathRef) -> NodeIndex {
        if let Some(&ndx) = self.fg.atom_map.get(&path_ref) {
            ndx
        } else {
            let ndx = self.fg.graph.add_node(path_ref.clone());
            self.fg.atom_map.insert(path_ref, ndx);
            ndx
        }
    }
    fn slot_with_path(&mut self, slot: Slot, path: Path) -> NodeIndex {
        let path_ref = PathRef {
            object: self.object,
            slot,
            path,
        };
        self.node(path_ref)
    }
    fn add_edge(&mut self, from: NodeIndex, to: NodeIndex, lop: &LocatedOpCode) {
        self.fg
            .graph
            .add_edge(from, to, EdgeKind::OpCode(lop.clone()));
    }
    /// [a.0, a.1, ...] --> [b.0, b.1, ...] (pairwise)
    fn assign(&mut self, from: Slot, to: Slot, lop: &LocatedOpCode) -> Result<(), RHDLError> {
        self.assign_from_to_path(from, Path::default(), to, Path::default(), lop)
    }
    /// [a.0, a.1, ...] --> [b.0, b.1, ...] (cross product) so all possible
    /// atoms of a are connected to all atoms of b.  
    fn splat(&mut self, from: Slot, to: Slot, lop: &LocatedOpCode) {
        let from_kind = self.object.kind(from);
        let to_kind = self.object.kind(to);
        for from_slot_path in from_kind.all_leafs() {
            for to_slot_path in to_kind.all_leafs() {
                let from_ndx = self.slot_with_path(from, from_slot_path.clone());
                let to_ndx = self.slot_with_path(to, to_slot_path);
                self.add_edge(from_ndx, to_ndx, lop);
            }
        }
    }
    /// [a.0, a.1, ...] --> [b.to_path.0, b.to_path.1, ...] (pairwise)
    /// We update a part of the destination path, so we need to convert b.to_path
    /// into an atom path and connect a's atoms to the corresponding sub-atoms of b.
    fn assign_to_path(
        &mut self,
        from: Slot,
        to: Slot,
        to_base_path: Path,
        lop: &LocatedOpCode,
    ) -> Result<(), RHDLError> {
        self.assign_from_to_path(from, Path::default(), to, to_base_path, lop)
    }
    /// [a.from_path.0, a.from_path.1, ...] --> [b.to_path.0, b.to_path.1, ...] (pairwise)
    /// We update a part of the source path and a part of the destination path, so we need to convert both paths
    /// into atom paths and connect the corresponding sub-atoms of a to the corresponding sub-atoms of b.
    fn assign_from_to_path(
        &mut self,
        from: Slot,
        from_base_path: Path,
        to: Slot,
        to_base_path: Path,
        lop: &LocatedOpCode,
    ) -> Result<(), RHDLError> {
        // The Kind of the the slot we are copying from.
        let from_kind = self.object.kind(from);
        // The Kind of the the subset of the slot we are copying.
        let from_source_kind = from_kind.sub_kind(&from_base_path)?;
        // The Kind of the the slot we are copying to.
        let to_kind = self.object.kind(to);
        // The Kind of the the subset of the slot we are copying to.
        let to_dest_kind = to_kind.sub_kind(&to_base_path)?;
        if from_source_kind != to_dest_kind {
            return Err(self.raise_error(
                FlowGraphErrorKind::CannotCopyAtomsFromPathToPath {
                    from: from_kind,
                    to: to_kind,
                    from_base_path,
                    to_base_path,
                    sub_kind: from_source_kind,
                    to_sub_kind: to_dest_kind,
                },
                lop.loc,
            ));
        }
        let from_paths = from_kind.leaf_paths(from_base_path.clone());
        let to_paths = to_kind.leaf_paths(to_base_path.clone());
        for (from_atom, to_atom) in from_paths.into_iter().zip(to_paths.into_iter()) {
            let from_ndx = self.slot_with_path(from, from_atom);
            let to_ndx = self.slot_with_path(to, to_atom);
            self.add_edge(from_ndx, to_ndx, lop);
        }
        Ok(())
    }
}

pub(crate) fn build_flow_graph(object: Intern<Object>) -> Result<FlowGraph, RHDLError> {
    // Import the arguments
    let mut builder = FlowGraphBuilder::new(object);
    // Create entries for all the leaves in the input arguments
    for arg in object.arguments.iter() {
        for atom in object.symtab[*arg].all_leafs() {
            let _ = builder.slot_with_path(arg.into(), atom);
        }
    }
    for atom in object.kind(object.return_slot).all_leafs() {
        let _ = builder.slot_with_path(object.return_slot, atom);
    }
    for lop in object.ops.iter() {
        match &lop.op {
            OpCode::Noop => {}
            // Binary operations will mix changes from either argument
            // into the destination value.  So a splat is OK here.
            OpCode::Binary(binary) => {
                builder.splat(binary.arg1, binary.lhs, lop);
                builder.splat(binary.arg2, binary.lhs, lop);
            }
            OpCode::Unary(unary) => {
                builder.splat(unary.arg1, unary.lhs, lop);
            }
            // Changes in the output can come from either the input
            // conditions changing, or the selector changing.
            OpCode::Select(select) => {
                builder.splat(select.cond, select.lhs, lop);
                builder.assign(select.true_value, select.lhs, lop)?;
                builder.assign(select.false_value, select.lhs, lop)?;
            }
            OpCode::Index(index) => {
                // Splat every dynamic index to the lhs
                for &dyn_index in index.path.dynamic_slots() {
                    builder.splat(dyn_index, index.lhs, lop);
                }
                let source_kind = builder.object.kind(index.arg);
                for path in path_star(source_kind, &index.path)? {
                    builder.assign_from_to_path(
                        index.arg,
                        path,
                        index.lhs,
                        Path::default(),
                        lop,
                    )?;
                }
            }
            OpCode::Assign(assign) => {
                // Straight assignment
                builder.assign(assign.rhs, assign.lhs, lop)?;
            }
            OpCode::Splice(splice) => {
                builder.assign(splice.orig, splice.lhs, lop)?;
                // Splat every dynamic index in the path to the lhs
                for &dyn_index in splice.path.dynamic_slots() {
                    builder.splat(dyn_index, splice.lhs, lop);
                }
                let source_kind = builder.object.kind(splice.orig);
                for path in path_star(source_kind, &splice.path)? {
                    builder.assign_from_to_path(
                        splice.subst,
                        Path::default(),
                        splice.lhs,
                        path.clone(),
                        lop,
                    )?;
                }
            }
            // Fan out of changes - if the value changes, all of the
            // fields in the output will change.
            OpCode::Repeat(repeat) => {
                for i in 0..repeat.len {
                    builder.assign_to_path(
                        repeat.value,
                        repeat.lhs,
                        Path::default().index(i as usize),
                        lop,
                    )?;
                }
            }
            // Take changes from the input for the fields provided.
            // Take remaining changes from the default value if provided.
            OpCode::Struct(strukt) => {
                let Kind::Struct(str_kind) = object.kind(strukt.lhs) else {
                    panic!("Expected struct kind");
                };
                let mut covered = str_kind
                    .fields
                    .iter()
                    .map(|m| m.name)
                    .collect::<HashSet<_>>();
                for field in &strukt.fields {
                    covered.remove(&field.member.to_string().into());
                    builder.assign_to_path(
                        field.value,
                        strukt.lhs,
                        Path::default().field(&field.member.to_string()),
                        lop,
                    )?;
                }
                if let Some(def) = strukt.rest {
                    for field in str_kind.fields.iter().filter(|f| covered.contains(&f.name)) {
                        builder.assign_from_to_path(
                            def,
                            Path::default().field(&field.name.to_string()),
                            strukt.lhs,
                            Path::default().field(&field.name.to_string()),
                            lop,
                        )?;
                    }
                }
            }
            // Each element of the tuple changes only if the corresponding input element changes.
            OpCode::Tuple(tuple) => {
                for (index, arg) in tuple.fields.iter().enumerate() {
                    builder.assign_to_path(
                        *arg,
                        tuple.lhs,
                        Path::default().tuple_index(index),
                        lop,
                    )?;
                }
            }
            // The output of a case can change if the discriminant changes or if any of the inputs to the selected case change.
            OpCode::Case(case) => {
                builder.splat(case.discriminant, case.lhs, lop);
                for (_arg, entry) in &case.table {
                    builder.assign(*entry, case.lhs, lop)?;
                }
            }
            // Exec is basically a set of copy-in and copy-out.  But it is special cased to deal with
            // the fact that the subfuction's flow graph needs to be imported and remapped.
            OpCode::Exec(exec) => {
                let sub_func = object.externals[&exec.id];
                let sub_fg = build_flow_graph(sub_func)?;
                let remap = builder.import(&sub_fg);
                // The copy to/from logic is duplicated here, because we are copying
                // across function boundaries.
                for (i, &arg) in exec.args.iter().enumerate() {
                    let sub_arg = sub_func.arguments[i];
                    let from_kind = builder.object.kind(arg);
                    let to_kind = sub_func.kind(sub_arg.into());
                    if from_kind != to_kind {
                        return Err(builder.raise_error(
                            FlowGraphErrorKind::MismatchedKindsInExternalFunctionCall {
                                expected: sub_func
                                    .arguments
                                    .iter()
                                    .map(|&a| sub_func.kind(a.into()))
                                    .collect(),
                                actual: exec.args.iter().map(|&a| builder.object.kind(a)).collect(),
                            },
                            lop.loc,
                        ));
                    }
                    for path in from_kind.all_leafs() {
                        // This is the node for the atom in our function
                        let from_ndx = builder.slot_with_path(arg, path.clone());
                        // Lookup the corresponding node in the sub function's flow graph and remap it to our flow graph
                        let to_ndx = sub_fg.lookup(sub_func, sub_arg.into(), path, lop.loc)?;
                        let to_ndx = remap[&to_ndx];
                        builder.add_edge(from_ndx, to_ndx, lop);
                    }
                }
                // Handle the return type
                let my_return_kind = builder.object.kind(exec.lhs);
                let their_return_kind = sub_func.kind(sub_func.return_slot);
                if my_return_kind != their_return_kind {
                    return Err(builder.raise_error(
                        FlowGraphErrorKind::MismatchedReturnKindsInExternalFunctionCall {
                            expected: their_return_kind,
                            actual: my_return_kind,
                        },
                        lop.loc,
                    ));
                }
                for path in my_return_kind.all_leafs() {
                    let from_ndx = builder.slot_with_path(exec.lhs, path.clone());
                    let to_ndx = sub_fg.lookup(sub_func, sub_func.return_slot, path, lop.loc)?;
                    let to_ndx = remap[&to_ndx];
                    builder.add_edge(to_ndx, from_ndx, lop);
                }
            }
            // Arrays are like tuples.  Each element of the output array changes only if the corresponding input element changes.
            OpCode::Array(array) => {
                // Changing arg # i --> lhs[i]
                for (index, element) in array.elements.iter().enumerate() {
                    builder.assign_to_path(
                        *element,
                        array.lhs,
                        Path::default().index(index),
                        lop,
                    )?;
                }
            }
            OpCode::Enum(enumerate) => {
                let discriminant = enumerate.template.discriminant()?.as_i64()?;
                for field in &enumerate.fields {
                    builder.assign_to_path(
                        field.value,
                        enumerate.lhs,
                        Path::default()
                            .payload_by_value(discriminant)
                            .member(&field.member),
                        lop,
                    )?;
                }
            }
            // Casts only operate on atoms, but they can change the Kind.  So we use a splat.
            OpCode::AsBits(cast) | OpCode::AsSigned(cast) | OpCode::Resize(cast) => {
                builder.splat(cast.arg, cast.lhs, lop);
            }
            // A retime is wrapping a value into a Signal struct.  The change in the target is
            // the value of the signal path.
            OpCode::Retime(retime) => {
                // Changing arg --> lhs
                builder.assign_to_path(
                    retime.arg,
                    retime.lhs,
                    Path::default().signal_value(),
                    lop,
                )?
            }
            // A wrap  modifies only the payload of the target, not the discriminant.  So we
            // tie the change to only that part of the target's path.
            OpCode::Wrap(wrap) => match wrap.op {
                WrapOp::Ok => {
                    builder.assign_to_path(
                        wrap.arg,
                        wrap.lhs,
                        Path::default().payload_by_value(1).tuple_index(0),
                        lop,
                    )?;
                }
                WrapOp::Some => builder.assign_to_path(
                    wrap.arg,
                    wrap.lhs,
                    Path::default().payload_by_value(1).tuple_index(0),
                    lop,
                )?,
                WrapOp::Err => {
                    builder.assign_to_path(
                        wrap.arg,
                        wrap.lhs,
                        Path::default().payload_by_value(0).tuple_index(0),
                        lop,
                    )
                }?,
                WrapOp::None => {}
            },
        }
    }
    Ok(builder.fg)
}

#[cfg(test)]
mod tests {
    use crate::{
        Kind,
        common::{slot_vec::SlotVec, symtab::RegisterId},
        rhif::spec::SlotKind,
        types::path::Path,
    };

    #[test]
    fn test_path_star() {
        // Create a Kind that is a
        // struct me {
        //  foo: [
        //     struct bar {
        //         baz: [(b8, b8); 2],
        //    },
        //  ; 2],
        // }
        let kind = Kind::make_struct(
            "me",
            vec![Kind::make_field(
                "foo",
                Kind::make_array(
                    Kind::make_struct(
                        "bar",
                        vec![Kind::make_field(
                            "baz",
                            Kind::make_array(
                                Kind::make_tuple(
                                    vec![Kind::make_bits(8), Kind::make_bits(8)].into(),
                                ),
                                2,
                            ),
                        )]
                        .into(),
                    ),
                    2,
                ),
            )]
            .into(),
        );
        let mut reg_map = SlotVec::<(), RegisterId<SlotKind>>::default();
        let i_slot = reg_map.push(());
        let j_slot = reg_map.push(());
        // Create a dynamic path that is foo[i].baz[j].0
        let path = Path::default()
            .field("foo")
            .dynamic(i_slot.into())
            .field("baz")
            .dynamic(j_slot.into())
            .tuple_index(0);
        // Call path_star on this path
        let paths = super::path_star(kind, &path).unwrap();
        // We should get back the following paths:
        // foo[0].baz[0].0
        // foo[0].baz[1].0
        // foo[1].baz[0].0
        // foo[1].baz[1].0
        for i in 0..2 {
            for j in 0..2 {
                let expected_path = Path::default()
                    .field("foo")
                    .index(i)
                    .field("baz")
                    .index(j)
                    .tuple_index(0);
                assert!(
                    paths.contains(&expected_path),
                    "Expected path {:?} not found in output paths",
                    expected_path
                );
            }
        }
    }
}
