use crate::{
    BitX, HDLDescriptor, Kind, RHDLError,
    ast::{SourceLocation, spanned_source::SpannedSourceSet},
    common::symtab::{RegisterId, SymbolTable},
    ntl::{
        hdl::build_hdl,
        spec::{OpCode, Wire, WireKind},
        visit::visit_object_wires_mut,
    },
    rhif::object::SourceDetails,
};
use std::hash::Hash;
use std::hash::Hasher;

use fnv::FnvHasher;

#[derive(Clone, Hash, PartialEq, Copy, Debug)]
pub enum BlackBoxMode {
    Synchronous,
    Asynchronous,
}

#[derive(Clone, Hash)]
pub struct BlackBox {
    pub code: HDLDescriptor,
    pub mode: BlackBoxMode,
}

#[derive(Clone, Hash)]
pub struct WireDetails {
    pub source_details: Option<SourceDetails>,
    pub kind: Kind,
    pub bit: usize,
}

#[derive(Clone, Default, Hash)]
pub struct Object {
    pub name: String,
    pub inputs: Vec<Vec<RegisterId<WireKind>>>,
    pub outputs: Vec<Wire>,
    pub ops: Vec<LocatedOpCode>,
    pub code: SpannedSourceSet,
    pub black_boxes: Vec<BlackBox>,
    pub symtab: SymbolTable<BitX, (), WireDetails, WireKind>,
}

impl Object {
    /// Link another netlist, and return the offset added
    /// to registers
    pub fn import(&mut self, other: &Object) -> impl Fn(Wire) -> Wire + use<> {
        let mut other = other.clone();
        let remap = self.symtab.merge(std::mem::take(&mut other.symtab));
        visit_object_wires_mut(&mut other, |_sense, wire| *wire = remap(*wire));
        // Fix up black box references
        let bb_offset = self.black_boxes.len();
        for lop in other.ops.iter_mut() {
            if let OpCode::BlackBox(blackbox) = &mut lop.op {
                blackbox.code = blackbox.code.offset(bb_offset);
            }
        }
        self.ops.extend(other.ops);
        self.code.extend(other.code.sources.clone());
        self.black_boxes.extend(other.black_boxes.clone());
        remap
    }
    pub fn hash_value(&self) -> u64 {
        let mut hasher = FnvHasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }
    pub fn bitx(&self, wire: Wire) -> Option<BitX> {
        wire.lit().map(|lid| self.symtab[lid])
    }
    pub fn as_vlog(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        build_hdl(name, self)
    }
}

#[derive(Clone, Hash)]
pub struct LocatedOpCode {
    pub op: OpCode,
    pub loc: Option<SourceLocation>,
}
