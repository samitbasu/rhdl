use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    Kind, RHDLError,
    ast::ast_impl::WrapOp,
    circuit::schematic::{Schematic, builder::SchematicBuilder, canonicalize_path},
    common::sense::Sense,
    error::rhdl_error,
    rhif::{
        self,
        object::LocatedOpCode,
        spec::{AluUnary, OpCode, Slot},
        visit::visit_slots,
    },
    types::path::{Path, PathElement, PathError, PathExt, sub_kind},
};

struct OpCodeBuilder<'a> {
    sources: &'a mut HashMap<Slot, usize>,
    object: &'a rhif::Object,
    child_index: usize,
    lop: &'a LocatedOpCode,
    builder: SchematicBuilder,
    my_inputs: HashMap<Slot, usize>,
}

impl<'a> OpCodeBuilder<'a> {
    fn assign_from_to_path(
        &mut self,
        from_slot: Slot,
        from_base_path: &Path,
        to_slot: Slot,
        to_base_path: &Path,
    ) -> Result<(), RHDLError> {
        log::debug!(
            "Assigning from slot {:?}[{:?}] path {:?} to slot {:?}[{:?}] path {:?}",
            from_slot,
            self.object.kind(from_slot),
            from_base_path,
            to_slot,
            self.object.kind(to_slot),
            to_base_path
        );
        // The kind of the slot we're assigning from
        let from_kind = self.object.kind(from_slot);
        // The kind of the subset of the slot we are copying
        let from_source_kind = from_kind.sub_kind(from_base_path)?;
        // The kind of the slot we're assigning to
        let to_kind = self.object.kind(to_slot);
        let to_dest_kind = to_kind.sub_kind(to_base_path)?;
        assert_eq!(from_source_kind, to_dest_kind);
        // We are copying something like a.foo -> b.1
        // where a composite item of type T is being copied.
        // So we need to connect all leaf paths of T
        // but with different prefixes
        if to_dest_kind.is_empty() {
            return Ok(());
        }
        log::debug!(
            "dest_kind is {:?}, with leafs {:#?}",
            to_dest_kind,
            to_dest_kind.all_leafs()
        );
        for path in to_dest_kind.all_leafs() {
            let source_path = from_base_path.join(&path);
            let dest_path = to_base_path.join(&path);
            let source_path = canonicalize_path(from_kind, &source_path)?;
            let dest_path = canonicalize_path(to_kind, &dest_path)?;
            log::debug!(
                "Assign from input {} path {:?} to output path {:?}",
                self.my_inputs[&from_slot],
                source_path,
                dest_path
            );
            self.builder
                .assign_from_to_path(self.my_inputs[&from_slot], source_path, dest_path)?;
        }
        Ok(())
    }
    fn build(mut self) -> Result<(Vec<Slot>, Schematic), RHDLError> {
        let my_inputs = &mut self.my_inputs;
        let mut op_inputs = vec![];
        visit_slots(&self.lop.op, |sense, &slot| match sense {
            Sense::Read => {
                log::debug!("Read slot {:?} of kind {:?}", slot, self.object.kind(slot));
                let port_index = self
                    .builder
                    .allocate_input_port(self.object.kind(slot), Some(slot));
                log::debug!("Allocated input port {} for slot {:?}", port_index, slot);
                my_inputs.insert(slot, port_index);
                op_inputs.push(slot);
            }
            Sense::Write => {
                log::debug!("Write slot {:?} of kind {:?}", slot, self.object.kind(slot));
                self.builder
                    .add_output_port(self.object.kind(slot), Some(slot))
                    .expect("Output ports should never fail to be added");
                self.sources.insert(slot, self.child_index);
            }
        });
        self.builder.with_name(&format!("{:?}", self.lop.op));
        match &self.lop.op {
            OpCode::Noop => {}
            OpCode::Binary(binary) => {
                self.builder
                    .splat_input_port_to_output(my_inputs[&binary.arg1])
                    .splat_input_port_to_output(my_inputs[&binary.arg2]);
            }
            OpCode::Unary(unary) => {
                if unary.op == AluUnary::Val {
                    // unary.arg1 is of kind signal(x)
                    // unary.lhs is of kind x
                    self.assign_from_to_path(
                        unary.arg1,
                        &Path::default().signal_value(),
                        unary.lhs,
                        &Path::default(),
                    )?;
                } else {
                    self.builder
                        .splat_input_port_to_output(my_inputs[&unary.arg1]);
                }
            }
            OpCode::Select(select) => {
                self.builder
                    .splat_input_port_to_output(my_inputs[&select.cond])
                    .assign_input_to_output(my_inputs[&select.true_value])
                    .assign_input_to_output(my_inputs[&select.false_value]);
            }
            OpCode::Index(index) => {
                for dyn_slot in index.path.dynamic_slots() {
                    self.builder.splat_input_port_to_output(my_inputs[dyn_slot]);
                }
                let source_kind = self.object.kind(index.arg);
                for path in path_star(source_kind, &index.path)? {
                    log::debug!(
                        "Assigning from {:?}[{:?}] to {:?}[{:?}] for path {:?}",
                        index.arg,
                        self.object.kind(index.arg),
                        index.lhs,
                        self.object.kind(index.lhs),
                        path
                    );
                    self.assign_from_to_path(index.arg, &path, index.lhs, &Path::default())?;
                }
            }
            OpCode::Assign(assign) => {
                self.builder.assign_input_to_output(my_inputs[&assign.rhs]);
            }
            OpCode::Splice(splice) => {
                let source_kind = self.object.kind(splice.orig);
                let mut path_sets = source_kind.all_leafs().into_iter().collect::<HashSet<_>>();
                // Splat every dynamic index in the path to the lhs
                for &dyn_index in splice.path.dynamic_slots() {
                    self.builder
                        .splat_input_port_to_output(my_inputs[&dyn_index]);
                }
                let subst_kind = self.object.kind(splice.subst);
                for path in path_star(source_kind, &splice.path)? {
                    for leaf in subst_kind.all_leafs() {
                        assert!(path_sets.remove(&path.join(&leaf)));
                    }
                    self.assign_from_to_path(splice.subst, &Path::default(), splice.lhs, &path)?;
                }
                for path in path_sets {
                    self.assign_from_to_path(splice.orig, &path, splice.lhs, &path)?;
                }
            }
            OpCode::Repeat(repeat) => {
                for i in 0..repeat.len {
                    self.assign_from_to_path(
                        repeat.value,
                        &Path::default(),
                        repeat.lhs,
                        &Path::default().index(i as usize),
                    )?
                }
            }
            OpCode::Struct(strukt) => {
                let Kind::Struct(str_kind) = self.object.kind(strukt.lhs) else {
                    panic!("Expected struct kind");
                };
                let mut covered = str_kind
                    .fields
                    .iter()
                    .map(|m| m.name)
                    .collect::<HashSet<_>>();
                for field in &strukt.fields {
                    covered.remove(&field.member.to_string().into());
                    self.assign_from_to_path(
                        field.value,
                        &Path::default(),
                        strukt.lhs,
                        &Path::default().field(&field.member.to_string()),
                    )?;
                }
                if let Some(def) = strukt.rest {
                    for field in str_kind.fields.iter().filter(|f| covered.contains(&f.name)) {
                        self.assign_from_to_path(
                            def,
                            &Path::default().field(&field.name.to_string()),
                            strukt.lhs,
                            &Path::default().field(&field.name.to_string()),
                        )?;
                    }
                }
            }
            OpCode::Tuple(tuple) => {
                for (index, arg) in tuple.fields.iter().enumerate() {
                    self.assign_from_to_path(
                        *arg,
                        &Path::default(),
                        tuple.lhs,
                        &Path::default().tuple_index(index),
                    )?;
                }
            }
            // The output of a case can change if the discriminant changes or if any of the inputs to the selected case change.
            OpCode::Case(case) => {
                self.builder
                    .splat_input_port_to_output(my_inputs[&case.discriminant]);
                for (_arg, entry) in &case.table {
                    self.builder.assign_input_to_output(my_inputs[entry]);
                }
            }
            // Exec is basically a set of copy-in and copy-out.  But it is special cased to deal with
            // the fact that the subfuction's flow graph needs to be imported and remapped.
            OpCode::Exec(exec) => {
                let sub_func = Arc::clone(&self.object.externals[&exec.id]);
                let child = build_schematic(sub_func)?;
                let child_index = self.builder.import(child);
                // The copy to/from logic is duplicated here, because we are copying
                // across function boundaries.
                (0..exec.args.len()).for_each(|i| {
                    self.builder.forward_input_to_child(i, child_index, i);
                });
                // Forward the child output to my output
                self.builder.forward_output_from_child(child_index);
            }
            // Arrays are like tuples.  Each element of the output array changes only if the corresponding input element changes.
            OpCode::Array(array) => {
                // Changing arg # i --> lhs[i]
                for (index, element) in array.elements.iter().enumerate() {
                    self.assign_from_to_path(
                        *element,
                        &Path::default(),
                        array.lhs,
                        &Path::default().index(index),
                    )?;
                }
            }
            OpCode::Enum(enumerate) => {
                let discriminant = enumerate.template.discriminant()?.as_i64()?;
                for field in &enumerate.fields {
                    self.assign_from_to_path(
                        field.value,
                        &Path::default(),
                        enumerate.lhs,
                        &Path::default()
                            .payload_by_value(discriminant)
                            .member(&field.member),
                    )?;
                }
            }
            // Casts only operate on atoms, but they can change the Kind.  So we use a splat.
            OpCode::AsBits(cast) | OpCode::AsSigned(cast) | OpCode::Resize(cast) => {
                self.builder
                    .splat_input_port_to_output(my_inputs[&cast.arg]);
            }
            // A retime is wrapping a value into a Signal struct.  The change in the target is
            // the value of the signal path.
            OpCode::Retime(retime) => {
                // Changing arg --> lhs
                self.assign_from_to_path(
                    retime.arg,
                    &Path::default(),
                    retime.lhs,
                    &Path::default().signal_value(),
                )?
            }
            // A wrap  modifies only the payload of the target, not the discriminant.  So we
            // tie the change to only that part of the target's path.
            OpCode::Wrap(wrap) => match wrap.op {
                WrapOp::Ok => {
                    self.assign_from_to_path(
                        wrap.arg,
                        &Path::default(),
                        wrap.lhs,
                        &Path::default().payload_by_value(1).tuple_index(0),
                    )?;
                }
                WrapOp::Some => self.assign_from_to_path(
                    wrap.arg,
                    &Path::default(),
                    wrap.lhs,
                    &Path::default().payload_by_value(1).tuple_index(0),
                )?,
                WrapOp::Err => {
                    self.assign_from_to_path(
                        wrap.arg,
                        &Path::default(),
                        wrap.lhs,
                        &Path::default().payload_by_value(0).tuple_index(0),
                    )?;
                }
                WrapOp::None => {}
            },
        }
        Ok((op_inputs, self.builder.build()))
    }
}

pub fn build_schematic(object: Arc<rhif::Object>) -> Result<Schematic, RHDLError> {
    let mut builder = SchematicBuilder::kernel();
    builder.with_name(&object.name);
    builder.with_filename(object.filename());
    builder.top_mut().type_name = object.type_name.to_string();
    // Import the arguments of the kernel as input ports
    for (index, arg) in object.arguments.iter().enumerate() {
        let arg_kind = object.kind(arg.into());
        builder.add_input_ports(index, arg_kind, Some(Slot::Register(*arg)))?;
    }
    // Import the return value of the kernel as output port
    let return_kind = object.kind(object.return_slot);
    builder.add_output_port(return_kind, Some(object.return_slot))?;
    let mut sources = HashMap::default();
    for (lit, (tb, _)) in object.symtab.iter_lit() {
        let mut lit_builder = SchematicBuilder::literal(lit.into(), tb);
        lit_builder.add_output_port(tb.kind(), Some(Slot::Literal(lit)))?;
        let child_index = builder.import(lit_builder.build());
        sources.insert(Slot::Literal(lit), child_index);
    }
    for lop in object.ops.iter() {
        log::debug!("Processing OpCode {:?} at location {:?}", lop.op, lop.loc);
        if matches!(lop.op, OpCode::Noop) {
            continue;
        }
        let (my_inputs, op_code_schematic) = OpCodeBuilder {
            sources: &mut sources,
            object: &object,
            child_index: builder.next_child_index(),
            lop,
            builder: SchematicBuilder::opcode(lop.loc),
            my_inputs: HashMap::default(),
        }
        .build()?;
        let child_index = builder.import(op_code_schematic);
        // Wire up the inputs to this opcode.
        for (index, input) in my_inputs.iter().enumerate() {
            if let Some(source_child) = sources.get(input) {
                builder.copy_child_output_to_child_input(*source_child, child_index, index);
            } else {
                // This must be an input argument
                let arg_index = object
                    .arguments
                    .iter()
                    .position(|&a| Slot::Register(a) == *input)
                    .unwrap();
                builder.forward_input_to_child(arg_index, child_index, index);
            }
        }
    }
    if let Some(source_child) = sources.get(&object.return_slot) {
        builder.forward_output_from_child(*source_child);
    } else {
        // This must be an input argument
        let arg_index = object
            .arguments
            .iter()
            .position(|&a| Slot::Register(a) == object.return_slot)
            .unwrap();
        builder.assign_input_to_output(arg_index);
    }
    Ok(builder.build())
}

// Enumerate all possible concrete paths that a dynamic path could take.
fn path_star(kind: Kind, path: &Path) -> Result<Vec<Path>, RHDLError> {
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
                    .for_each(|item| *item = prefix_path.join(item));
                return Ok(suffix_star);
            }
        }
    }
    Ok(vec![path.clone()])
}
