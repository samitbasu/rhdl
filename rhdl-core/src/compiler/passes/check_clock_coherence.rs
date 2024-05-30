use anyhow::{anyhow, bail};
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

use crate::{
    rhif::{
        spec::{OpCode, Slot},
        Object,
    },
    Color, Kind,
};
use anyhow::Result;

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

#[derive(Debug, Clone, Default)]
struct ColorMap {
    map: BTreeMap<Slot, SlotColor>,
}

impl ColorMap {
    fn get_color(&self, slot: Slot) -> Result<SlotColor> {
        self.map
            .get(&slot)
            .cloned()
            .ok_or_else(|| anyhow!("Slot {:?} not found", slot))
    }
    fn unify(&mut self, slot: Slot, color: SlotColor) -> Result<()> {
        if let Some(old_color) = self.map.insert(slot, color) {
            if color != get_merged_color([old_color, color]) {
                bail!("Slot {:?} has conflicting colors", slot);
            }
        }
        Ok(())
    }
    fn insert(&mut self, slot: Slot, color: SlotColor) -> Option<SlotColor> {
        self.map.insert(slot, color)
    }
}

impl Display for ColorMap {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for (slot, color) in self.map.iter() {
            writeln!(f, "{:?} -> {:?}", slot, color)?;
        }
        Ok(())
    }
}

pub struct CheckClockCoherence {}

impl Pass for CheckClockCoherence {
    fn name(&self) -> &'static str {
        "check_clock_coherence"
    }
    fn description(&self) -> &'static str {
        "Check that all clocked signals are coherent"
    }
    fn run(input: Object) -> Result<Object> {
        check_clock_coherence(&input)?;
        Ok(input)
    }
}

fn check_clock_coherence(obj: &Object) -> Result<()> {
    let mut map: ColorMap = Default::default();
    // Next, populate the map with the information from the type map in the
    // object, by presuming that all registers have been properly typed
    for (slot, kind) in obj.kind.iter() {
        map.insert(*slot, get_slot_color_for_kind(kind));
    }
    // Apply coherence rules to binary ops.
    for op in obj.ops.iter() {
        match op {
            OpCode::Binary(binary) => {
                let arg1_color = map.get_color(binary.arg1)?;
                let arg2_color = map.get_color(binary.arg2)?;
                let lhs_color = get_merged_color([arg1_color, arg2_color]);
                map.unify(binary.lhs, lhs_color)?;
            }
            OpCode::Unary(unary) => {
                let arg_color = map.get_color(unary.arg1)?;
                map.unify(unary.lhs, arg_color)?;
            }
            OpCode::Select(select) => {
                let true_color = map.get_color(select.true_value)?;
                map.unify(select.cond, true_color)?;
                let false_color = map.get_color(select.false_value)?;
                map.unify(select.cond, false_color)?;
                let cond_color = map.get_color(select.cond)?;
                let lhs_color = get_merged_color([cond_color, true_color, false_color]);
                map.unify(select.lhs, lhs_color)?;
            }
            OpCode::Index(index) => {
                // For the magic ".#val" index, the color is propagated
                // from the argument.
                if index.path.is_magic_val_path() {
                    let base_color = map.get_color(index.arg)?;
                    map.unify(index.lhs, base_color)?;
                }
            }
            OpCode::Case(case) => {
                let result_color = map.get_color(case.lhs)?;
                map.unify(case.discriminant, result_color)?;
            }
            _ => {}
        }
    }
    // Print it for now
    eprintln!("{}", map);
    Ok(())
}
