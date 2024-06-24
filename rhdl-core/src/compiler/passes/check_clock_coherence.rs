use std::collections::{BTreeMap, HashSet};

use crate::{
    ast::ast_impl::NodeId,
    compiler::mir::{
        error::{ClockError, RHDLClockCoherenceViolation},
        ty::{make_variant_tag, AppType, AppTypeKind, Const, TypeId, TypeKind, UnifyContext},
    },
    error::RHDLError,
    rhif::{
        spec::{OpCode, Slot},
        Object,
    },
    types::path::{Path, PathElement},
    Color, Kind,
};

use super::pass::Pass;

pub struct CheckClockCoherence {}

impl Pass for CheckClockCoherence {
    fn name() -> &'static str {
        "check_clock_coherence"
    }
    fn description() -> &'static str {
        "Check that all clocked signals are coherent"
    }
    fn run(input: Object) -> Result<Object, RHDLError> {
        check_clock_coherence(&input)?;
        Ok(input)
    }
}

struct ClockCoherenceContext<'a> {
    obj: &'a Object,
    ctx: UnifyContext,
    slot_map: BTreeMap<Slot, TypeId>,
}

impl ClockCoherenceContext<'_> {
    fn collect_clock_domains(&mut self, ty: TypeId) -> Vec<TypeId> {
        match self.ctx.ty(ty) {
            TypeKind::Var(_) => vec![ty],
            TypeKind::App(AppType { kind: _, args }) => args
                .clone()
                .into_iter()
                .flat_map(|arg| self.collect_clock_domains(arg))
                .collect(),
            TypeKind::Const(Const::Clock(_)) => vec![ty],
            _ => vec![],
        }
    }
    fn clock_domain_for_error(&mut self, ty: TypeId) -> &'static str {
        let domains = self
            .collect_clock_domains(ty)
            .into_iter()
            .map(|ty| {
                if let Ok(clock) = self.ctx.cast_ty_as_clock(ty) {
                    match clock {
                        Color::Red => "Red",
                        Color::Orange => "Orange",
                        Color::Yellow => "Yellow",
                        Color::Green => "Green",
                        Color::Blue => "Blue",
                        Color::Indigo => "Indigo",
                        Color::Violet => "Violet",
                    }
                } else {
                    "Unknown"
                }
            })
            .collect::<HashSet<_>>();
        if domains.len() == 1 {
            domains.into_iter().next().unwrap()
        } else {
            "Multiple"
        }
    }
    fn raise_clock_coherence_error(
        &mut self,
        containing_id: NodeId,
        slots: &[Slot],
        cause: ClockError,
    ) -> Box<RHDLClockCoherenceViolation> {
        // Print out the current state of the slot map
        eprintln!("Slot map:");
        for (slot, ty) in &self.slot_map {
            let ty = self.ctx.apply(*ty);
            eprintln!("{:?} -> {:?}", slot, self.ctx.desc(ty));
        }
        let elements = slots
            .iter()
            .copied()
            .map(|slot| {
                let ty = self.slot_map[&slot];
                let ty = self.ctx.apply(ty);
                (
                    format!(
                        "Expression belongs to {} clock domain",
                        self.clock_domain_for_error(ty)
                    ),
                    self.obj
                        .symbols
                        .best_span_for_slot_in_expression(slot, containing_id)
                        .into(),
                )
            })
            .collect();

        eprintln!(
            "cause span: {:?}",
            self.obj.symbols.node_span(containing_id)
        );
        Box::new(RHDLClockCoherenceViolation {
            src: self.obj.symbols.source.source.clone(),
            elements,
            cause,
            cause_span: self.obj.symbols.node_span(containing_id).into(),
        })
    }
    // Here, domain is the clock domain for all of the fields in the kind that is being
    // imported.
    fn import_kind_with_single_domain(
        &mut self,
        id: NodeId,
        kind: &Kind,
        domain: TypeId,
    ) -> TypeId {
        match kind {
            Kind::Array(array) => {
                let base = self.import_kind_with_single_domain(id, &array.base, domain);
                let len = self.ctx.ty_const_len(id, array.size);
                self.ctx.ty_array(id, base, len)
            }
            Kind::Tuple(tuple) => {
                let elements = tuple
                    .elements
                    .iter()
                    .map(|kind| self.import_kind_with_single_domain(id, kind, domain))
                    .collect();
                self.ctx.ty_tuple(id, elements)
            }
            Kind::Struct(strukt) => {
                let fields = strukt
                    .fields
                    .iter()
                    .map(|field| {
                        let tid = self.import_kind_with_single_domain(id, &field.kind, domain);
                        (field.name.clone(), tid)
                    })
                    .collect();
                self.ctx.ty_dyn_struct(id, strukt.name.clone(), fields)
            }
            Kind::Enum(enumerate) => {
                let layout = enumerate.discriminant_layout;
                let name = enumerate.name.clone();
                let variants = enumerate
                    .variants
                    .iter()
                    .map(|variant| {
                        let ty = self.import_kind_with_single_domain(id, &variant.kind, domain);
                        let tag = make_variant_tag(&variant.name, variant.discriminant, variant.ty);
                        (tag, ty)
                    })
                    .collect();
                self.ctx.ty_dyn_enum(id, name, layout, variants)
            }
            Kind::Bits(_) | Kind::Signed(_) | Kind::Empty => domain,
            Kind::Signal(base, clock) => {
                let clock = self.ctx.ty_clock(id, *clock);
                self.import_kind_with_single_domain(id, base, clock)
            }
        }
    }
    fn import_kind_with_unknown_domains(&mut self, id: NodeId, kind: &Kind) -> TypeId {
        match kind {
            Kind::Tuple(tuple) => {
                let elements = tuple
                    .elements
                    .iter()
                    .map(|kind| self.import_kind_with_unknown_domains(id, kind))
                    .collect();
                self.ctx.ty_tuple(id, elements)
            }
            Kind::Struct(strukt) => {
                let fields = strukt
                    .fields
                    .iter()
                    .map(|field| {
                        let tid = self.import_kind_with_unknown_domains(id, &field.kind);
                        (field.name.clone(), tid)
                    })
                    .collect();
                self.ctx.ty_dyn_struct(id, strukt.name.clone(), fields)
            }
            Kind::Signal(base, color) => {
                let domain = self.ctx.ty_clock(id, *color);
                self.import_kind_with_single_domain(id, base, domain)
            }
            _ => {
                let domain = self.ctx.ty_var(id);
                self.import_kind_with_single_domain(id, kind, domain)
            }
        }
    }
    fn import_literals(&mut self) {
        for (slot, literal) in &self.obj.literals {
            eprintln!("Importing literal {:?} {:?}", slot, literal.kind);
            let id = self.obj.symbols.slot_map[slot].node;
            let ty = self.import_kind_with_unknown_domains(id, &literal.kind);
            self.slot_map.insert(*slot, ty);
        }
    }
    fn import_registers(&mut self) {
        for (slot, kind) in &self.obj.kind {
            let id = self.obj.symbols.slot_map[slot].node;
            let ty = self.import_kind_with_unknown_domains(id, kind);
            self.slot_map.insert(*slot, ty);
        }
    }
    fn unify_clocks(
        &mut self,
        slots: &[Slot],
        cause_id: NodeId,
        cause: ClockError,
    ) -> Result<(), RHDLError> {
        let ty_clock = self.ctx.ty_var(cause_id);
        for slot in slots {
            if !slot.is_empty() {
                let ty_slot = self.slot_map[slot];
                if self.ctx.unify(ty_slot, ty_clock).is_err() {
                    return Err(self
                        .raise_clock_coherence_error(cause_id, slots, cause)
                        .into());
                }
            }
        }
        Ok(())
    }

    fn ty_path_project(
        &mut self,
        arg_slot: Slot,
        path: &Path,
        id: NodeId,
    ) -> Result<TypeId, RHDLError> {
        let arg_ty = self.slot_map[&arg_slot];
        let mut arg = self.ctx.apply(arg_ty);
        for element in path.elements.iter() {
            eprintln!("Path project {} {:?}", self.ctx.desc(arg), element);
            match element {
                PathElement::Index(ndx) => {
                    arg = self.ctx.ty_index(arg, *ndx)?;
                }
                PathElement::Field(member) => {
                    arg = self.ctx.ty_field(arg, member)?;
                }
                PathElement::EnumDiscriminant => {
                    arg = self.ctx.ty_enum_discriminant(arg)?;
                }
                PathElement::TupleIndex(ndx) => {
                    arg = self.ctx.ty_index(arg, *ndx)?;
                }
                PathElement::EnumPayload(member) => {
                    arg = self.ctx.ty_variant(arg, member)?;
                }
                PathElement::DynamicIndex(slot) => {
                    eprintln!("Dynamic index {:?} {:?}", slot, arg_slot);
                    // First, index the argument type to get a base type
                    arg = self.ctx.ty_index(arg, 0)?;
                    let slot_ty = self.slot_map[slot];
                    if self.ctx.unify(arg, slot_ty).is_err() {
                        return Err(self
                            .raise_clock_coherence_error(
                                id,
                                &[arg_slot, *slot],
                                ClockError::IndexClockMismatch,
                            )
                            .into());
                    }
                }
                PathElement::EnumPayloadByValue(value) => {
                    arg = self.ctx.ty_variant_by_value(arg, *value)?;
                }
                PathElement::SignalValue => {
                    todo!()
                    /*                     arg = self
                                           .ctx
                                           .project_signal_value(arg)
                                           .ok_or(self.raise_type_error(TypeCheck::ExpectedSignalValue, id))?;
                    */
                }
            }
        }
        Ok(arg)
    }
    fn dump_resolution(&mut self) {
        let resolved_map = self
            .slot_map
            .clone()
            .into_iter()
            .map(|(k, v)| (k, self.ctx.apply(v)))
            .collect::<Vec<_>>();
        for ty in &resolved_map {
            let desc = self.ctx.desc(ty.1);
            eprintln!("Slot {:?} has type {:?}", ty.0, desc);
        }
    }
    fn check(&mut self) -> Result<(), RHDLError> {
        eprintln!("Code before clock check: {:?}", self.obj);
        self.import_literals();
        self.import_registers();
        self.dump_resolution();
        let mut active_set = HashSet::new();
        active_set.insert(self.obj.return_slot);
        active_set.extend(self.obj.arguments.iter());
        for (op, location) in self.obj.ops.iter().zip(self.obj.symbols.opcode_map.iter()) {
            let id = location.node;
            if !matches!(op, OpCode::Noop) {
                eprintln!("Check clock coherence for {:?}", op);
            }
            match op {
                OpCode::Binary(binary) => {
                    self.unify_clocks(
                        &[binary.arg1, binary.arg2, binary.lhs],
                        location.node,
                        ClockError::BinaryOperationClockMismatch { op: binary.op },
                    )?;
                    active_set.insert(binary.arg1);
                    active_set.insert(binary.arg2);
                }
                OpCode::Unary(unary) => {
                    self.unify_clocks(
                        &[unary.arg1, unary.lhs],
                        location.node,
                        ClockError::UnaryOperationClockMismatch { op: unary.op },
                    )?;
                    active_set.insert(unary.arg1);
                }
                OpCode::Assign(assign) => {
                    self.unify_clocks(
                        &[assign.rhs, assign.lhs],
                        location.node,
                        ClockError::AssignmentClockMismatch,
                    )?;
                    active_set.insert(assign.rhs);
                }
                OpCode::AsBits(cast) | OpCode::AsSigned(cast) => {
                    self.unify_clocks(
                        &[cast.arg, cast.lhs],
                        location.node,
                        ClockError::CastClockMismatch,
                    )?;
                    active_set.insert(cast.arg);
                }
                OpCode::Retime(retime) => {
                    self.unify_clocks(
                        &[retime.arg, retime.lhs],
                        location.node,
                        ClockError::RetimeClockMismatch,
                    )?;
                    active_set.insert(retime.arg);
                }
                OpCode::Select(select) => {
                    // Create a type with the clock of the condition slot, but with
                    // the kind structure of the lhs.
                    // The select argument must be a boolean scalar, which is mapped to a
                    // single clock domain.
                    let domain = self.slot_map[&select.cond];
                    let lhs_id = self.obj.symbols.slot_map[&select.lhs].node;
                    let lhs_ty = self.import_kind_with_single_domain(
                        lhs_id,
                        &self.obj.kind[&select.lhs],
                        domain,
                    );
                    eprintln!(
                        "Unify select lhs {} and {}",
                        self.ctx.desc(lhs_ty),
                        self.ctx.desc(self.slot_map[&select.lhs])
                    );
                    if self.ctx.unify(lhs_ty, self.slot_map[&select.lhs]).is_err()
                        || self
                            .ctx
                            .unify(lhs_ty, self.slot_map[&select.true_value])
                            .is_err()
                        || self
                            .ctx
                            .unify(lhs_ty, self.slot_map[&select.false_value])
                            .is_err()
                    {
                        return Err(self
                            .raise_clock_coherence_error(
                                location.node,
                                &[
                                    select.lhs,
                                    select.cond,
                                    select.true_value,
                                    select.false_value,
                                ],
                                ClockError::SelectClockMismatch,
                            )
                            .into());
                    }
                    active_set.extend([select.cond, select.true_value, select.false_value].iter());
                }
                OpCode::Index(index) => {
                    let rhs_project =
                        self.ty_path_project(index.arg, &index.path, location.node)?;
                    let ty_lhs = self.slot_map[&index.lhs];
                    if self.ctx.unify(rhs_project, ty_lhs).is_err() {
                        return Err(self
                            .raise_clock_coherence_error(
                                location.node,
                                &[index.arg, index.lhs],
                                ClockError::IndexClockMismatch,
                            )
                            .into());
                    }
                    active_set.insert(index.arg);
                    active_set.extend(index.path.dynamic_slots());
                }
                OpCode::Splice(_) => todo!(),
                OpCode::Repeat(_) => todo!(),
                OpCode::Struct(_) => todo!(),
                OpCode::Tuple(_) => todo!(),
                OpCode::Case(_) => todo!(),
                OpCode::Exec(_) => todo!(),
                OpCode::Array(_) => todo!(),
                OpCode::Enum(_) => todo!(),
                OpCode::Comment(_) | OpCode::Noop => {}
            }
        }
        eprintln!("*****Clock coherence check complete*****");
        for slot in &active_set {
            eprintln!("Active slot {:?}", slot);
        }
        let resolved_map = self
            .slot_map
            .clone()
            .into_iter()
            .filter(|(k, _)| active_set.contains(k))
            .map(|(k, v)| (k, self.ctx.apply(v)))
            .collect::<Vec<_>>();
        for ty in &resolved_map {
            let desc = self.ctx.desc(ty.1);
            eprintln!("Slot {:?} has type {:?}", ty.0, desc);
            if self.ctx.is_unresolved(ty.1) {
                return Err(self
                    .raise_clock_coherence_error(
                        self.obj.symbols.source.fallback,
                        &[ty.0],
                        ClockError::UnresolvedClock,
                    )
                    .into());
            }
        }
        Ok(())
    }
}

fn check_clock_coherence(obj: &Object) -> Result<(), RHDLError> {
    ClockCoherenceContext {
        obj,
        ctx: UnifyContext::default(),
        slot_map: BTreeMap::new(),
    }
    .check()
}
