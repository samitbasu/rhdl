use log::debug;
use std::collections::{BTreeMap, HashSet};

use crate::{
    ast::source::source_location::SourceLocation,
    compiler::mir::{
        error::{ClockError, RHDLClockDomainViolation},
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

pub struct CheckClockDomain {}

impl Pass for CheckClockDomain {
    fn run(input: Object) -> Result<Object, RHDLError> {
        check_clock_domain(&input)?;
        Ok(input)
    }
}

struct ClockDomainContext<'a> {
    obj: &'a Object,
    ctx: UnifyContext,
    slot_map: BTreeMap<Slot, TypeId>,
}

impl ClockDomainContext<'_> {
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
    fn raise_clock_domain_error(
        &mut self,
        containing_id: SourceLocation,
        slots: &[Slot],
        cause: ClockError,
    ) -> Box<RHDLClockDomainViolation> {
        // Print out the current state of the slot map
        debug!("Slot map:");
        for (slot, ty) in &self.slot_map {
            let ty = self.ctx.apply(*ty);
            debug!("{:?} -> {:?}", slot, self.ctx.desc(ty));
        }
        let elements = slots
            .iter()
            .copied()
            .map(|slot| {
                let ty = self.slot_type(&slot);
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

        Box::new(RHDLClockDomainViolation {
            src: self.obj.symbols.source(),
            elements,
            cause,
            cause_span: self.obj.symbols.span(containing_id).into(),
        })
    }
    // Here, domain is the clock domain for all of the fields in the kind that is being
    // imported.
    fn import_kind_with_single_domain(
        &mut self,
        id: SourceLocation,
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
                        let tag = make_variant_tag(&variant.name, variant.discriminant);
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
    fn import_kind_with_unknown_domains(&mut self, id: SourceLocation, kind: &Kind) -> TypeId {
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
        for (&lit_id, _) in &self.obj.literals {
            let id = self.obj.symbols.slot_map[&lit_id.into()];
            let ty = self.ctx.ty_unclocked(id);
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
        cause_id: SourceLocation,
        cause: ClockError,
    ) -> Result<(), RHDLError> {
        debug!("unify clocks {:?}", slots);
        let ty_clock = self.ctx.ty_var(cause_id);
        for slot in slots {
            if !slot.is_empty() {
                let ty_slot = self.slot_type(slot);
                if self.ctx.unify(ty_slot, ty_clock).is_err() {
                    return Err(self.raise_clock_domain_error(cause_id, slots, cause).into());
                }
            }
        }
        debug!("After unify clocks");
        for slot in slots {
            let ty_slot = self.slot_type(slot);
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
        cause_id: SourceLocation,
        cause: ClockError,
    ) -> Result<(), RHDLError> {
        let lhs_ty = self.slot_type(&lhs);
        let lhs_domains = self.collect_clock_domains(lhs_ty);
        for lhs_domain in lhs_domains {
            let rhs_ty = self.slot_type(&rhs);
            let rhs_domains = self.collect_clock_domains(rhs_ty);
            for domain in rhs_domains {
                if self.ctx.unify(lhs_domain, domain).is_err() {
                    return Err(self
                        .raise_clock_domain_error(cause_id, &[lhs, rhs], cause)
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
        id: SourceLocation,
    ) -> Result<TypeId, RHDLError> {
        let arg_ty = self.slot_type(&arg_slot);
        let mut arg = self.ctx.apply(arg_ty);
        for element in path.elements.iter() {
            debug!("Path project {} {:?}", self.ctx.desc(arg), element);
            match element {
                PathElement::Index(ndx) => {
                    arg = self
                        .ctx
                        .ty_index(arg, *ndx)
                        .unwrap_or_else(|_| self.ctx.ty_var(id));
                }
                PathElement::Field(member) => {
                    arg = self
                        .ctx
                        .ty_field(arg, member)
                        .unwrap_or_else(|_| self.ctx.ty_var(id));
                }
                PathElement::EnumDiscriminant => arg = self.ctx.ty_enum_discriminant(arg),
                PathElement::TupleIndex(ndx) => {
                    arg = self
                        .ctx
                        .ty_index(arg, *ndx)
                        .unwrap_or_else(|_| self.ctx.ty_var(id));
                }
                PathElement::EnumPayload(member) => {
                    arg = self
                        .ctx
                        .ty_variant(arg, member)
                        .unwrap_or_else(|_| self.ctx.ty_var(id));
                }
                PathElement::DynamicIndex(slot) => {
                    debug!("Dynamic index {:?} {:?}", slot, arg_slot);
                    let slot_ty = self.slot_type(slot);
                    if self.ctx.is_unresolved(slot_ty) || self.ctx.is_unresolved(arg_ty) {
                        return Ok(self.ctx.ty_var(id));
                    }
                    // First, index the argument type to get a base type
                    arg = self.ctx.ty_index(arg, 0)?;
                    let slot_domains = self.collect_clock_domains(slot_ty);
                    let arg_domains = self.collect_clock_domains(arg);
                    for arg_domain in arg_domains {
                        for slot_domain in &slot_domains {
                            if self.ctx.unify(arg_domain, *slot_domain).is_err() {
                                return Err(self
                                    .raise_clock_domain_error(
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
                    arg = self
                        .ctx
                        .ty_variant_by_value(arg, *value)
                        .unwrap_or_else(|_| self.ctx.ty_var(id));
                }
                PathElement::SignalValue => {}
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
            debug!("Slot {:?} has type {:?}", ty.0, desc);
        }
    }
    fn slot_type(&mut self, slot: &Slot) -> TypeId {
        // If the slot is a literal or empty, just make up a new
        // variable and return it.
        match slot {
            Slot::Empty => {
                let id = self.obj.symbols.slot_map[slot];
                self.ctx.ty_unclocked(id)
            }
            Slot::Register(_) | Slot::Literal(_) => self.slot_map[slot],
        }
    }
    fn check(&mut self) -> Result<(), RHDLError> {
        debug!("Code before clock check: {:?}", self.obj);
        self.import_literals();
        self.import_registers();
        self.dump_resolution();
        for lop in &self.obj.ops {
            let op = &lop.op;
            let loc = lop.loc;
            if !matches!(op, OpCode::Noop) {
                debug!("Check clock domain for {:?}", op);
            }
            match op {
                OpCode::Binary(binary) => {
                    if !binary.op.is_comparison() {
                        self.unify_clocks(
                            &[binary.arg1, binary.arg2, binary.lhs],
                            loc,
                            ClockError::BinaryOperationClockMismatch { op: binary.op },
                        )?;
                    } else {
                        self.unify_clocks(
                            &[binary.arg1, binary.arg2],
                            loc,
                            ClockError::BinaryOperationClockMismatch { op: binary.op },
                        )?;
                        self.unify_projected_clocks(
                            binary.arg1,
                            binary.lhs,
                            loc,
                            ClockError::BinaryOperationClockMismatch { op: binary.op },
                        )?;
                    }
                }
                OpCode::Unary(unary) => {
                    if matches!(unary.op, AluUnary::Neg | AluUnary::Not) {
                        self.unify_clocks(
                            &[unary.arg1, unary.lhs],
                            loc,
                            ClockError::UnaryOperationClockMismatch { op: unary.op },
                        )?;
                    } else {
                        self.unify_projected_clocks(
                            unary.arg1,
                            unary.lhs,
                            loc,
                            ClockError::UnaryOperationClockMismatch { op: unary.op },
                        )?;
                    }
                }
                OpCode::Assign(assign) => {
                    self.unify_clocks(
                        &[assign.rhs, assign.lhs],
                        loc,
                        ClockError::AssignmentClockMismatch,
                    )?;
                }
                OpCode::AsBits(cast) | OpCode::AsSigned(cast) | OpCode::Resize(cast) => {
                    self.unify_clocks(&[cast.arg, cast.lhs], loc, ClockError::CastClockMismatch)?;
                }
                OpCode::Retime(retime) => {
                    self.unify_clocks(
                        &[retime.arg, retime.lhs],
                        loc,
                        ClockError::RetimeClockMismatch,
                    )?;
                }
                OpCode::Select(select) => {
                    self.unify_projected_clocks(
                        select.cond,
                        select.lhs,
                        loc,
                        ClockError::SelectClockMismatch,
                    )?;
                    self.unify_clocks(
                        &[select.lhs, select.true_value, select.false_value],
                        loc,
                        ClockError::SelectClockMismatch,
                    )?;
                }
                OpCode::Wrap(wrap) => {
                    self.unify_projected_clocks(
                        wrap.lhs,
                        wrap.arg,
                        loc,
                        ClockError::WrapClockMismatch,
                    )?;
                }
                OpCode::Index(index) => {
                    let rhs_project = self.ty_path_project(index.arg, &index.path, loc)?;
                    let ty_lhs = self.slot_type(&index.lhs);
                    if self.ctx.unify(rhs_project, ty_lhs).is_err() {
                        return Err(self
                            .raise_clock_domain_error(
                                loc,
                                &[index.arg, index.lhs],
                                ClockError::IndexClockMismatch,
                            )
                            .into());
                    }
                }
                OpCode::Splice(splice) => {
                    let lhs_ty = self.slot_type(&splice.lhs);
                    let orig_ty = self.slot_type(&splice.orig);
                    let subst_ty = self.slot_type(&splice.subst);
                    let path_ty = self.ty_path_project(splice.orig, &splice.path, loc)?;
                    debug!("lhs_ty: {:?}", self.ctx.desc(lhs_ty));
                    debug!("orig_ty: {:?}", self.ctx.desc(orig_ty));
                    debug!("subst_ty: {:?}", self.ctx.desc(subst_ty));
                    debug!("path_ty: {:?}", self.ctx.desc(path_ty));
                    if self.ctx.unify(orig_ty, lhs_ty).is_err() {
                        return Err(self
                            .raise_clock_domain_error(
                                loc,
                                &[splice.lhs, splice.orig],
                                ClockError::SpliceClockMismatch,
                            )
                            .into());
                    }
                    if self.ctx.unify(subst_ty, path_ty).is_err() {
                        return Err(self
                            .raise_clock_domain_error(
                                loc,
                                &[splice.subst, splice.orig],
                                ClockError::SpliceClockMismatch,
                            )
                            .into());
                    }
                    debug!("After unify");
                    let lhs_ty = self.ctx.apply(lhs_ty);
                    let orig_ty = self.ctx.apply(orig_ty);
                    let subst_ty = self.ctx.apply(subst_ty);
                    let path_ty = self.ctx.apply(path_ty);
                    debug!("lhs_ty: {:?}", self.ctx.desc(lhs_ty));
                    debug!("orig_ty: {:?}", self.ctx.desc(orig_ty));
                    debug!("subst_ty: {:?}", self.ctx.desc(subst_ty));
                    debug!("path_ty: {:?}", self.ctx.desc(path_ty));
                }
                OpCode::Array(array) => {
                    let ty_lhs = self.slot_type(&array.lhs);
                    for element in &array.elements {
                        let ty_len = self.ctx.ty_var(loc);
                        let ty_rhs = self.slot_type(element);
                        let rhs = self.ctx.ty_array(loc, ty_rhs, ty_len);
                        if self.ctx.unify(ty_lhs, rhs).is_err() {
                            return Err(self
                                .raise_clock_domain_error(
                                    loc,
                                    &[array.lhs, *element],
                                    ClockError::ArrayClockMismatch,
                                )
                                .into());
                        }
                    }
                }
                OpCode::Repeat(repeat) => {
                    let ty_lhs = self.slot_type(&repeat.lhs);
                    let ty_rhs = self.slot_type(&repeat.value);
                    let ty_len = self.ctx.ty_const_len(loc, repeat.len as usize);
                    let rhs = self.ctx.ty_array(loc, ty_rhs, ty_len);
                    if self.ctx.unify(ty_lhs, rhs).is_err() {
                        return Err(self
                            .raise_clock_domain_error(
                                loc,
                                &[repeat.lhs, repeat.value],
                                ClockError::ArrayClockMismatch,
                            )
                            .into());
                    }
                }
                OpCode::Struct(strukt) => {
                    let ty_lhs = self.slot_type(&strukt.lhs);
                    for field in &strukt.fields {
                        let ty_rhs = self.slot_type(&field.value);
                        let ty_field = self.ctx.ty_member(ty_lhs, &field.member)?;
                        if self.ctx.unify(ty_field, ty_rhs).is_err() {
                            return Err(self
                                .raise_clock_domain_error(
                                    loc,
                                    &[strukt.lhs, field.value],
                                    ClockError::StructClockMismatch,
                                )
                                .into());
                        }
                    }
                }
                OpCode::Tuple(tuple) => {
                    let ty_lhs = self.slot_type(&tuple.lhs);
                    let ty_rhs_elements = tuple
                        .fields
                        .iter()
                        .map(|field| self.slot_type(field))
                        .collect::<Vec<_>>();
                    let ty_rhs = self.ctx.ty_tuple(loc, ty_rhs_elements);
                    let tl = self.ctx.apply(ty_lhs);
                    let tr = self.ctx.apply(ty_rhs);
                    debug!("Tuple {:?} U {:?}", self.ctx.desc(tl), self.ctx.desc(tr));
                    if self.ctx.unify(ty_lhs, ty_rhs).is_err() {
                        let slots = std::iter::once(tuple.lhs)
                            .chain(tuple.fields.iter().copied())
                            .collect::<Vec<_>>();
                        return Err(self
                            .raise_clock_domain_error(loc, &slots, ClockError::TupleClockMismatch)
                            .into());
                    }
                }
                OpCode::Case(case) => {
                    self.unify_projected_clocks(
                        case.discriminant,
                        case.lhs,
                        loc,
                        ClockError::CaseClockMismatch,
                    )?;
                    for (argument, value) in &case.table {
                        self.unify_projected_clocks(
                            case.discriminant,
                            *value,
                            loc,
                            ClockError::CaseClockMismatch,
                        )?;
                        if let CaseArgument::Slot(slot) = argument {
                            self.unify_projected_clocks(
                                case.discriminant,
                                *slot,
                                loc,
                                ClockError::CaseClockMismatch,
                            )?;
                        }
                    }
                }
                OpCode::Exec(exec) => {
                    let sub = &self.obj.externals[&exec.id];
                    let ret_ty =
                        self.import_kind_with_unknown_domains(loc, &sub.kind(sub.return_slot));
                    let lhs_ty = self.slot_type(&exec.lhs);
                    if self.ctx.unify(ret_ty, lhs_ty).is_err() {
                        return Err(self
                            .raise_clock_domain_error(
                                loc,
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
                        let arg_ty = self.import_kind_with_unknown_domains(loc, &kind);
                        let slot_ty = self.slot_type(slot);
                        if self.ctx.unify(arg_ty, slot_ty).is_err() {
                            return Err(self
                                .raise_clock_domain_error(
                                    loc,
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
                            loc,
                            ClockError::EnumClockMismatch,
                        )?;
                    }
                }
                OpCode::Comment(_) | OpCode::Noop => {}
            }
        }
        debug!("*****Clock domain check complete*****");
        let resolved_map = self
            .slot_map
            .clone()
            .into_iter()
            .map(|(k, v)| (k, self.ctx.apply(v)))
            .collect::<Vec<_>>();
        for ty in &resolved_map {
            let desc = self.ctx.desc(ty.1);
            debug!("Slot {:?} has type {:?}", ty.0, desc);
            if self.ctx.is_unresolved(ty.1) {
                return Err(self
                    .raise_clock_domain_error(
                        self.obj.symbols.fallback(self.obj.fn_id),
                        &[ty.0],
                        ClockError::UnresolvedClock,
                    )
                    .into());
            }
        }
        Ok(())
    }
}

fn check_clock_domain(obj: &Object) -> Result<(), RHDLError> {
    ClockDomainContext {
        obj,
        ctx: UnifyContext::default(),
        slot_map: BTreeMap::new(),
    }
    .check()
}
