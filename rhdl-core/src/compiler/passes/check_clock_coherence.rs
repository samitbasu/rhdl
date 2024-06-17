use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

use serde::de;

use crate::{
    ast::ast_impl::NodeId,
    compiler::mir::{
        error::{ClockError, RHDLClockCoherenceViolation, ICE},
        ty::{AppTypeKind, StructType, TypeId, UnifyContext},
    },
    error::{rhdl_error, RHDLError},
    rhif::{
        spec::{OpCode, Slot},
        Object,
    },
    types::path::{Path, PathElement},
    Color, Kind,
};

use super::pass::Pass;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SlotColor {
    Multicolor,
    Single(Color),
    Uncolored,
}

fn get_merged_color(seq: impl IntoIterator<Item = SlotColor>) -> SlotColor {
    seq.into_iter()
        .fold(SlotColor::Uncolored, |acc, color| match (acc, color) {
            (SlotColor::Uncolored, color) => color,
            (color, SlotColor::Uncolored) => color,
            (SlotColor::Single(color1), SlotColor::Single(color2)) if color1 == color2 => {
                SlotColor::Single(color1)
            }
            _ => SlotColor::Multicolor,
        })
}

fn get_slot_color_for_kind(kind: &Kind) -> SlotColor {
    match kind {
        Kind::Array(array) => get_slot_color_for_kind(&array.base),
        Kind::Tuple(tuple) => get_merged_color(tuple.elements.iter().map(get_slot_color_for_kind)),
        Kind::Struct(structure) => get_merged_color(
            structure
                .fields
                .iter()
                .map(|field| get_slot_color_for_kind(&field.kind)),
        ),
        Kind::Enum(enumerate) => get_merged_color(
            enumerate
                .variants
                .iter()
                .map(|variant| get_slot_color_for_kind(&variant.kind)),
        ),
        Kind::Signal(_, color) => SlotColor::Single(*color),
        Kind::Bits(_) | Kind::Signed(_) | Kind::Empty => SlotColor::Uncolored,
    }
}

#[derive(Debug)]
struct ColorMap<'a> {
    obj: &'a Object,
    map: BTreeMap<Slot, SlotColor>,
}

impl<'a> ColorMap<'a> {
    fn get_color(&self, slot: Slot) -> Result<SlotColor, RHDLError> {
        self.map.get(&slot).cloned().ok_or_else(|| {
            CheckClockCoherence::raise_ice(
                &self.obj,
                ICE::MissingSlotInColorMap { slot },
                self.obj.symbols.slot_map[&slot].node,
            )
        })
    }
    fn unify(&mut self, slot: Slot, color: SlotColor) -> Result<(), RHDLError> {
        if let Some(prev_color) = self.map.get(&slot) {
            let new_color = get_merged_color([*prev_color, color]);
            if new_color == SlotColor::Multicolor {
                return Err(CheckClockCoherence::raise_ice(
                    self.obj,
                    ICE::SlotHasConflictingColors { slot },
                    self.obj.symbols.slot_map[&slot].node,
                ));
            }
            self.map.insert(slot, new_color);
        } else {
            self.map.insert(slot, color);
        }
        Ok(())
    }
    fn insert(&mut self, slot: Slot, color: SlotColor) -> Option<SlotColor> {
        self.map.insert(slot, color)
    }
}

impl<'a> Display for ColorMap<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for (slot, color) in self.map.iter() {
            writeln!(f, "{:?} -> {:?}", slot, color)?;
        }
        Ok(())
    }
}

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
    fn clock_domain_for_error(&mut self, ty: TypeId) -> &'static str {
        let Some(ty) = self.ctx.project_signal_clock(ty) else {
            return "Unresolved";
        };
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
            "Const"
        }
    }
    fn raise_clock_coherence_error(
        &mut self,
        id: NodeId,
        slots: &[Slot],
        cause: ClockError,
    ) -> Box<RHDLClockCoherenceViolation> {
        let elements = slots
            .iter()
            .copied()
            .map(|slot| {
                let ty = self.slot_map[&slot];
                let id = self.obj.symbols.slot_map[&slot].node;
                (
                    format!(
                        "Expression belongs to {} clock domain",
                        self.clock_domain_for_error(ty)
                    ),
                    self.obj.symbols.node_span(id).into(),
                )
            })
            .collect();
        Box::new(RHDLClockCoherenceViolation {
            src: self.obj.symbols.source.source.clone(),
            elements,
            cause,
            cause_span: self.obj.symbols.node_span(id).into(),
        })
    }
    fn wrap_kind_in_signal(&mut self, base: TypeId, id: NodeId) -> TypeId {
        let clock = self.ctx.ty_var(id);
        self.ctx.ty_signal(id, base, clock)
    }
    // Promotes a base kind T to a signal of type <T, C>, with an undetermined domain C.
    fn promote_kind_to_timed(&mut self, id: NodeId, kind: &Kind) -> TypeId {
        let base = self.ctx.from_kind(id, kind);
        match kind {
            Kind::Empty => self.ctx.ty_empty(id),
            Kind::Signal(_, _) => base,
            Kind::Tuple(tuple) => {
                let elements = tuple
                    .elements
                    .iter()
                    .map(|kind| self.promote_kind_to_timed(id, kind))
                    .collect();
                self.ctx.ty_tuple(id, elements)
            }
            Kind::Struct(strukt) => {
                let fields = strukt
                    .fields
                    .iter()
                    .map(|field| {
                        let tid = self.promote_kind_to_timed(id, &field.kind);
                        (field.name.clone(), tid)
                    })
                    .collect();
                self.ctx.ty_dyn_struct(id, strukt.name.clone(), fields)
            }
            _ => self.wrap_kind_in_signal(base, id),
        }
    }
    fn import_literals(&mut self) {
        for (slot, literal) in &self.obj.literals {
            eprintln!("Importing literal {:?} {:?}", slot, literal.kind);
            let id = self.obj.symbols.slot_map[slot].node;
            let ty = self.promote_kind_to_timed(id, &literal.kind);
            self.slot_map.insert(*slot, ty);
        }
    }
    fn import_registers(&mut self) {
        for (slot, kind) in &self.obj.kind {
            let id = self.obj.symbols.slot_map[slot].node;
            let ty = self.promote_kind_to_timed(id, kind);
            self.slot_map.insert(*slot, ty);
        }
    }
    fn unify_clocks(
        &mut self,
        slots: &[Slot],
        id: NodeId,
        cause: ClockError,
    ) -> Result<(), RHDLError> {
        let ty_clock = self.ctx.ty_var(id);
        for slot in slots {
            let id = self.obj.symbols.slot_map[slot].node;
            let ty_slot = self.slot_map[slot];
            let projected = self.ctx.project_clocks(ty_slot);
            let projected = projected
                .iter()
                .map(|clock| self.ctx.desc(*clock))
                .collect::<Vec<_>>();
            eprintln!(
                "Checking slot {:?} {:?} -> {:?}",
                slot,
                self.ctx.desc(ty_slot),
                projected
            );
            for clock in self.ctx.project_clocks(ty_slot) {
                eprintln!(
                    "Checking clock {:?} {:?}",
                    self.ctx.desc(clock),
                    self.ctx.desc(ty_clock)
                );
                if self.ctx.unify(clock, ty_clock).is_err() {
                    return Err(self.raise_clock_coherence_error(id, slots, cause).into());
                }
            }
        }
        Ok(())
    }

    fn ty_path_project(
        &mut self,
        arg: TypeId,
        path: &Path,
        id: NodeId,
    ) -> Result<TypeId, RHDLError> {
        let mut arg = self.ctx.apply(arg);
        for element in path.elements.iter() {
            eprintln!("Path project {:?} {:?}", arg, element);
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
                    todo!()
                    /*                     let index = self.slot_ty(*slot);
                                       let usize_ty = self.ctx.ty_usize(id);
                                       if slot.is_literal() {
                                           self.unify(id, index, usize_ty)?;
                                       } else {
                                           let reg_ty = self.ctx.apply(index);
                                           if self.ctx.is_generic_integer(reg_ty) {
                                               // For more clearly defined types, it is someone else's problem
                                               // to ensure that the index is properly typed.
                                               self.unify(id, reg_ty, usize_ty)?;
                                           }
                                       }
                                       arg = self.ctx.ty_index(arg, 0)?;
                    */
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

    fn check(&mut self) -> Result<(), RHDLError> {
        self.import_literals();
        self.import_registers();
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
                }
                OpCode::Unary(unary) => {
                    self.unify_clocks(
                        &[unary.arg1, unary.lhs],
                        location.node,
                        ClockError::UnaryOperationClockMismatch { op: unary.op },
                    )?;
                }
                OpCode::Assign(assign) => {
                    self.unify_clocks(
                        &[assign.rhs, assign.lhs],
                        location.node,
                        ClockError::AssignmentClockMismatch,
                    )?;
                }
                OpCode::Retime(retime) => {
                    self.unify_clocks(
                        &[retime.arg, retime.lhs],
                        location.node,
                        ClockError::RetimeClockMismatch,
                    )?;
                }
                OpCode::Select(select) => {
                    self.unify_clocks(
                        &[
                            select.cond,
                            select.lhs,
                            select.true_value,
                            select.false_value,
                        ],
                        location.node,
                        ClockError::SelectClockMismatch,
                    )?;
                }
                OpCode::Index(index) => {
                    let ty_arg = self.slot_map[&index.arg];
                    let rhs_project = self.ty_path_project(ty_arg, &index.path, location.node)?;
                    let ty_lhs = self.slot_map[&index.lhs];
                    self.ctx.unify(rhs_project, ty_lhs).unwrap();
                    if self.ctx.unify(rhs_project, ty_lhs).is_err() {
                        return Err(self
                            .raise_clock_coherence_error(
                                location.node,
                                &[index.arg, index.lhs],
                                ClockError::IndexClockMismatch,
                            )
                            .into());
                    }
                }
                _ => {}
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
            for clock in self.ctx.project_clocks(ty.1) {
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
