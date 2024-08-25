use std::collections::{BTreeMap, HashSet};

use crate::{
    ast::ast_impl::NodeId,
    compiler::mir::{
        error::{ClockError, RHDLClockCoherenceViolation},
        ty::{make_variant_tag, AppTypeKind, Const, TypeId, TypeKind, UnifyContext},
    },
    error::RHDLError,
    rhif::{
        spec::{AluUnary, CaseArgument, OpCode, Slot},
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
            TypeKind::App(app) => app
                .sub_types()
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
                let discriminant = domain;
                self.ctx.ty_dyn_enum(
                    id,
                    name,
                    discriminant,
                    enumerate.discriminant_layout.alignment,
                    variants,
                )
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
        for (&lit_id, literal) in &self.obj.literals {
            eprintln!("Importing literal {:?} {:?}", lit_id, literal.kind);
            let id = self.obj.symbols.slot_map[&lit_id.into()];
            let ty = self.import_kind_with_unknown_domains(id, &literal.kind);
            self.slot_map.insert(lit_id.into(), ty);
        }
    }
    fn import_registers(&mut self) {
        for (&reg_id, kind) in &self.obj.kind {
            let id = self.obj.symbols.slot_map[&reg_id.into()];
            let ty = self.import_kind_with_unknown_domains(id, kind);
            self.slot_map.insert(reg_id.into(), ty);
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
    // Unify the clock domains present in the RHS with the domain given on the LHS.
    // So, for example, if the LHS is Green, and RHS is struct<foo: V1, bar: V2>, then
    // V1 and V2 must be Green.
    fn unify_projected_clocks(
        &mut self,
        lhs: Slot,
        rhs: Slot,
        cause_id: NodeId,
        cause: ClockError,
    ) -> Result<(), RHDLError> {
        let lhs_domains = self.collect_clock_domains(self.slot_map[&lhs]);
        for lhs_domain in lhs_domains {
            let rhs_domains = self.collect_clock_domains(self.slot_map[&rhs]);
            for domain in rhs_domains {
                if self.ctx.unify(lhs_domain, domain).is_err() {
                    return Err(self
                        .raise_clock_coherence_error(cause_id, &[lhs, rhs], cause)
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
                    arg = self.ctx.ty_enum_discriminant(arg);
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
                    let slot_domains = self.collect_clock_domains(slot_ty);
                    let arg_domains = self.collect_clock_domains(arg);
                    for arg_domain in arg_domains {
                        for slot_domain in &slot_domains {
                            if self.ctx.unify(arg_domain, *slot_domain).is_err() {
                                return Err(self
                                    .raise_clock_coherence_error(
                                        id,
                                        &[arg_slot, *slot],
                                        ClockError::IndexClockMismatch,
                                    )
                                    .into());
                            }
                        }
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
        for lop in &self.obj.ops {
            let op = &lop.op;
            let id = lop.id;
            if !matches!(op, OpCode::Noop) {
                eprintln!("Check clock coherence for {:?}", op);
            }
            match op {
                OpCode::Binary(binary) => {
                    if !binary.op.is_comparison() {
                        self.unify_clocks(
                            &[binary.arg1, binary.arg2, binary.lhs],
                            lop.id,
                            ClockError::BinaryOperationClockMismatch { op: binary.op },
                        )?;
                    } else {
                        self.unify_clocks(
                            &[binary.arg1, binary.arg2],
                            lop.id,
                            ClockError::BinaryOperationClockMismatch { op: binary.op },
                        )?;
                        self.unify_projected_clocks(
                            binary.arg1,
                            binary.lhs,
                            lop.id,
                            ClockError::BinaryOperationClockMismatch { op: binary.op },
                        )?;
                    }
                }
                OpCode::Unary(unary) => {
                    if matches!(unary.op, AluUnary::Neg | AluUnary::Not) {
                        self.unify_clocks(
                            &[unary.arg1, unary.lhs],
                            lop.id,
                            ClockError::UnaryOperationClockMismatch { op: unary.op },
                        )?;
                    } else {
                        self.unify_projected_clocks(
                            unary.arg1,
                            unary.lhs,
                            lop.id,
                            ClockError::UnaryOperationClockMismatch { op: unary.op },
                        )?;
                    }
                }
                OpCode::Assign(assign) => {
                    self.unify_clocks(
                        &[assign.rhs, assign.lhs],
                        lop.id,
                        ClockError::AssignmentClockMismatch,
                    )?;
                }
                OpCode::AsBits(cast) | OpCode::AsSigned(cast) => {
                    self.unify_clocks(
                        &[cast.arg, cast.lhs],
                        lop.id,
                        ClockError::CastClockMismatch,
                    )?;
                }
                OpCode::Retime(retime) => {
                    self.unify_clocks(
                        &[retime.arg, retime.lhs],
                        lop.id,
                        ClockError::RetimeClockMismatch,
                    )?;
                }
                OpCode::Select(select) => {
                    self.unify_projected_clocks(
                        select.cond,
                        select.lhs,
                        lop.id,
                        ClockError::SelectClockMismatch,
                    )?;
                    self.unify_clocks(
                        &[select.lhs, select.true_value, select.false_value],
                        id,
                        ClockError::SelectClockMismatch,
                    )?;
                }
                OpCode::Index(index) => {
                    let rhs_project = self.ty_path_project(index.arg, &index.path, lop.id)?;
                    let ty_lhs = self.slot_map[&index.lhs];
                    if self.ctx.unify(rhs_project, ty_lhs).is_err() {
                        return Err(self
                            .raise_clock_coherence_error(
                                lop.id,
                                &[index.arg, index.lhs],
                                ClockError::IndexClockMismatch,
                            )
                            .into());
                    }
                }
                OpCode::Splice(splice) => {
                    let lhs_ty = self.slot_map[&splice.lhs];
                    let orig_ty = self.slot_map[&splice.orig];
                    let subst_ty = self.slot_map[&splice.subst];
                    let path_ty = self.ty_path_project(splice.orig, &splice.path, lop.id)?;
                    if self.ctx.unify(orig_ty, lhs_ty).is_err() {
                        return Err(self
                            .raise_clock_coherence_error(
                                lop.id,
                                &[splice.lhs, splice.orig],
                                ClockError::SpliceClockMismatch,
                            )
                            .into());
                    }
                    if self.ctx.unify(subst_ty, path_ty).is_err() {
                        return Err(self
                            .raise_clock_coherence_error(
                                lop.id,
                                &[splice.subst, splice.orig],
                                ClockError::SpliceClockMismatch,
                            )
                            .into());
                    }
                }
                OpCode::Array(array) => {
                    let ty_lhs = self.slot_map[&array.lhs];
                    for element in &array.elements {
                        let ty_len = self.ctx.ty_var(lop.id);
                        let ty_rhs = self.slot_map[element];
                        let rhs = self.ctx.ty_array(lop.id, ty_rhs, ty_len);
                        if self.ctx.unify(ty_lhs, rhs).is_err() {
                            return Err(self
                                .raise_clock_coherence_error(
                                    lop.id,
                                    &[array.lhs, *element],
                                    ClockError::ArrayClockMismatch,
                                )
                                .into());
                        }
                    }
                }
                OpCode::Repeat(repeat) => {
                    let ty_lhs = self.slot_map[&repeat.lhs];
                    let ty_rhs = self.slot_map[&repeat.value];
                    let ty_len = self.ctx.ty_const_len(lop.id, repeat.len as usize);
                    let rhs = self.ctx.ty_array(lop.id, ty_rhs, ty_len);
                    if self.ctx.unify(ty_lhs, rhs).is_err() {
                        return Err(self
                            .raise_clock_coherence_error(
                                lop.id,
                                &[repeat.lhs, repeat.value],
                                ClockError::ArrayClockMismatch,
                            )
                            .into());
                    }
                }
                OpCode::Struct(strukt) => {
                    let ty_lhs = self.slot_map[&strukt.lhs];
                    for field in &strukt.fields {
                        let ty_rhs = self.slot_map[&field.value];
                        let ty_field = self.ctx.ty_member(ty_lhs, &field.member)?;
                        if self.ctx.unify(ty_field, ty_rhs).is_err() {
                            return Err(self
                                .raise_clock_coherence_error(
                                    lop.id,
                                    &[strukt.lhs, field.value],
                                    ClockError::StructClockMismatch,
                                )
                                .into());
                        }
                    }
                }
                OpCode::Tuple(tuple) => {
                    let ty_lhs = self.slot_map[&tuple.lhs];
                    let ty_rhs_elements = tuple
                        .fields
                        .iter()
                        .map(|field| self.slot_map[field])
                        .collect::<Vec<_>>();
                    let ty_rhs = self.ctx.ty_tuple(lop.id, ty_rhs_elements);
                    if self.ctx.unify(ty_lhs, ty_rhs).is_err() {
                        return Err(self
                            .raise_clock_coherence_error(
                                lop.id,
                                &[tuple.lhs],
                                ClockError::TupleClockMismatch,
                            )
                            .into());
                    }
                }
                OpCode::Case(case) => {
                    self.unify_projected_clocks(
                        case.discriminant,
                        case.lhs,
                        lop.id,
                        ClockError::CaseClockMismatch,
                    )?;
                    for (argument, value) in &case.table {
                        self.unify_projected_clocks(
                            case.discriminant,
                            *value,
                            lop.id,
                            ClockError::CaseClockMismatch,
                        )?;
                        if let CaseArgument::Slot(slot) = argument {
                            self.unify_projected_clocks(
                                case.discriminant,
                                *slot,
                                lop.id,
                                ClockError::CaseClockMismatch,
                            )?;
                        }
                    }
                }
                OpCode::Exec(exec) => {
                    let sub = &self.obj.externals[&exec.id];
                    let ret_ty =
                        self.import_kind_with_unknown_domains(lop.id, &sub.kind(sub.return_slot));
                    if self.ctx.unify(ret_ty, self.slot_map[&exec.lhs]).is_err() {
                        return Err(self
                            .raise_clock_coherence_error(
                                lop.id,
                                &[exec.lhs],
                                ClockError::ExternalClockMismatch,
                            )
                            .into());
                    }
                    for (kind, slot) in sub
                        .arguments
                        .iter()
                        .map(|x| sub.kind(Slot::Register(*x)))
                        .zip(exec.args.iter())
                    {
                        let arg_ty = self.import_kind_with_unknown_domains(lop.id, &kind);
                        if self.ctx.unify(arg_ty, self.slot_map[slot]).is_err() {
                            return Err(self
                                .raise_clock_coherence_error(
                                    lop.id,
                                    &[*slot],
                                    ClockError::ExternalClockMismatch,
                                )
                                .into());
                        }
                    }
                }
                OpCode::Enum(enumerate) => {
                    for variant in &enumerate.fields {
                        self.unify_projected_clocks(
                            enumerate.lhs,
                            variant.value,
                            lop.id,
                            ClockError::EnumClockMismatch,
                        )?;
                    }
                }
                OpCode::Comment(_) | OpCode::Noop => {}
            }
        }
        eprintln!("*****Clock coherence check complete*****");
        let resolved_map = self
            .slot_map
            .clone()
            .into_iter()
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
