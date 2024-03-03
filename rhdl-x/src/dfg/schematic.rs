use std::slice::SliceIndex;

use rhdl_core::Kind;

use super::components::{BufferComponent, Component, ComponentKind};

#[derive(Clone, Debug, Copy)]
pub struct PinIx(usize);

impl std::fmt::Display for PinIx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "p{}", self.0)
    }
}

const ORPHAN: ComponentIx = ComponentIx(!0);

#[derive(Clone, Debug, Copy)]
pub struct ComponentIx(usize);

impl std::fmt::Display for ComponentIx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "c{}", self.0)
    }
}

// Need some fixup mechanism for this so that
// pins can be created, then components, and then
// the pins can be updated to point to the correct
// component.

#[derive(Debug, Clone)]
pub struct Pin {
    pub kind: Kind,
    pub name: String,
    pub parent: ComponentIx,
}

impl Pin {
    pub fn parent(&mut self, parent: ComponentIx) {
        self.parent = parent;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Wire {
    pub source: PinIx,
    pub dest: PinIx,
}

#[derive(Clone, Debug, Default)]
pub struct Schematic {
    pub pins: Vec<Pin>,
    pub components: Vec<Component>,
    pub wires: Vec<Wire>,
    pub inputs: Vec<PinIx>,
    pub outputs: Vec<PinIx>,
}

impl Schematic {
    pub fn make_pin(&mut self, kind: Kind, name: String) -> PinIx {
        let ix = PinIx(self.pins.len());
        self.pins.push(Pin {
            kind,
            name,
            parent: ORPHAN,
        });
        ix
    }
    pub fn make_component(&mut self, name: String, kind: ComponentKind) -> ComponentIx {
        let ix = ComponentIx(self.components.len());
        self.components.push(Component { name, kind });
        ix
    }
    pub fn pin(&self, ix: PinIx) -> &Pin {
        &self.pins[ix.0]
    }
    pub fn pin_mut(&mut self, ix: PinIx) -> &mut Pin {
        &mut self.pins[ix.0]
    }
    pub fn wire(&mut self, source: PinIx, dest: PinIx) {
        self.wires.push(Wire { source, dest });
    }
}
