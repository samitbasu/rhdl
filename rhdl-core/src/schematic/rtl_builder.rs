use std::collections::HashMap;

use crate::rtl::{spec::Operand, Object};

use super::rtl_schematic::{PinIx, Schematic};

pub struct SchematicBuilder<'a> {
    object: &'a Object,
    schematic: Schematic,
    operand_map: HashMap<Operand, PinIx>,
}
