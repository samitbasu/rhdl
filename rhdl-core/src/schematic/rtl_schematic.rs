use super::db::{ComponentIx, PinIx};

#[derive(Debug, Clone, Copy)]
pub struct Wire {
    pub source: PinIx,
    pub dest: PinIx,
}

#[derive(Clone, Debug)]
pub struct Schematic {
    pub components: Vec<ComponentIx>,
    pub wires: Vec<Wire>,
    pub inputs: Vec<PinIx>,
    pub output: PinIx,
}
