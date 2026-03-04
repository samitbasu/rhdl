#![allow(missing_docs)]
use std::collections::BTreeMap;

use internment::Intern;
use serde::{Deserialize, Serialize};

use rhdl_trace_type::TraceType;

use crate::{
    BitX, Kind, RHDLError,
    ast::{SourceLocation, SourcePool, spanned_source::SpannedSourceSet},
    circuit::schematic::error::SchematicICE,
    common::{sense::Sense, slot_vec::SlotKey},
    error::rhdl_error,
    rhif::{object::LocatedOpCode, visit::visit_slots},
    types::path::{Path, PathElement, sub_kind},
};
pub mod builder;
pub mod circuit;
pub mod connected_checks;
pub mod dot;
pub mod error;
pub mod id_checks;
pub mod kernel;
pub mod loop_check;
pub mod svg;
pub mod synchronous;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedBits {
    bits: Vec<BitX>,
    kind: TraceType,
}

impl From<&crate::types::typed_bits::TypedBits> for TypedBits {
    fn from(value: &crate::types::typed_bits::TypedBits) -> Self {
        Self {
            bits: value.bits().iter().copied().collect(),
            kind: value.kind().into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpCode {
    pub name: String,
    pub inputs: Vec<Slot>,
    pub outputs: Vec<Slot>,
    pub location: SourceLocation,
}

impl From<&LocatedOpCode> for OpCode {
    fn from(value: &LocatedOpCode) -> Self {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        visit_slots(&value.op, |sense, &slot| match sense {
            Sense::Read => inputs.push(slot.into()),
            Sense::Write => outputs.push(slot.into()),
        });
        Self {
            name: format!("{:?}", value.op),
            inputs,
            outputs,
            location: value.loc,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    pub name: String,
    pub literals: BTreeMap<usize, TypedBits>,
    pub registers: BTreeMap<usize, TraceType>,
    pub opcodes: Vec<OpCode>,
    pub source_pool: SourcePool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct SchematicSet {
    pub schematics: Schematic,
    pub loop_ports: Vec<PortId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SchematicKind {
    Circuit,
    Synchronous,
    Kernel,
    OpCode,
    Literal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Schematic {
    pub id: SchematicId,
    pub name: String,
    pub type_name: String,
    pub filename: String,
    pub debug_text: String,
    pub kind: SchematicKind,
    pub inputs: Vec<Vec<Port>>,
    pub outputs: Vec<Port>,
    pub inner: Vec<Schematic>,
    pub links: Vec<Link>,
    pub sources: Intern<SpannedSourceSet>,
    pub location: Option<SourceLocation>,
}

impl Schematic {
    pub fn max_port_id(&self) -> PortId {
        self.inputs
            .iter()
            .flatten()
            .chain(self.outputs.iter())
            .map(|p| p.id)
            .chain(self.inner.iter().map(|c| c.max_port_id()))
            .max()
            .unwrap_or(PortId(0))
    }
    pub fn shift_ports(&mut self, offset: PortId) {
        self.inputs
            .iter_mut()
            .flatten()
            .for_each(|port| port.id.shift(offset));
        self.outputs
            .iter_mut()
            .for_each(|port| port.id.shift(offset));
        self.inner
            .iter_mut()
            .for_each(|child| child.shift_ports(offset));
        for link in &mut self.links {
            link.from.shift(offset);
            link.to.shift(offset);
        }
    }
    pub fn max_schematic_id(&self) -> SchematicId {
        self.inner
            .iter()
            .map(|c| c.max_schematic_id())
            .chain(std::iter::once(self.id))
            .max()
            .unwrap_or(self.id)
    }
    pub fn shift_id(&mut self, offset: SchematicId) {
        self.id.shift(offset);
        self.inner
            .iter_mut()
            .for_each(|child| child.shift_id(offset));
    }
    pub fn to_svg(&self) -> ::svg::Document {
        svg::schematic(self).unwrap()
    }
    pub fn to_dot(&self) -> String {
        dot::to_dot(self)
    }
    pub fn checked(&self) -> Result<(), RHDLError> {
        id_checks::check_ids(self)?;
        connected_checks::check_connected(self)?;
        loop_check::loop_check(self)?;
        Ok(())
    }
    pub fn find_by_id(&self, id: SchematicId) -> Option<&Schematic> {
        if self.id == id {
            Some(self)
        } else {
            self.inner.iter().find_map(|child| child.find_by_id(id))
        }
    }
    pub fn check_for_combinatorial_io_paths(&self) -> Result<(), RHDLError> {
        loop_check::check_combinatorial_pathways(self)
    }
}

#[derive(
    Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, serde::Serialize, serde::Deserialize,
)]
pub struct PortId(u32);

impl PortId {
    pub fn shift(&mut self, offset: PortId) {
        self.0 += offset.0;
    }
    pub fn index(&self) -> u32 {
        self.0
    }
}

#[derive(
    Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, serde::Serialize, serde::Deserialize,
)]
pub struct SchematicId(u32);

impl SchematicId {
    pub fn shift(&mut self, offset: SchematicId) {
        self.0 += offset.0;
    }
    pub fn next(&self) -> SchematicId {
        SchematicId(self.0 + 1)
    }
}

impl From<u32> for SchematicId {
    fn from(value: u32) -> Self {
        SchematicId(value)
    }
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Slot {
    Register(usize),
    Literal(usize),
}

impl From<crate::rhif::spec::Slot> for Slot {
    fn from(value: crate::rhif::spec::Slot) -> Self {
        if let Some(lit) = value.lit() {
            return Self::Literal(lit.index());
        }
        if let Some(reg) = value.reg() {
            return Self::Register(reg.index());
        }
        panic!("Invalid slot: {:?}", value);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Port {
    pub path: CanonicalPath,
    pub bits: usize,
    pub ty: TraceType,
    pub id: PortId,
    pub slot: Option<Slot>,
}

pub fn port(
    base_kind: Kind,
    path: &Path,
    port_kind: Kind,
    id: PortId,
    slot: Option<crate::rhif::spec::Slot>,
) -> Result<Port, RHDLError> {
    let path = canonicalize_path(base_kind, path)?;
    let slot = slot.map(|x| x.into());
    Ok(Port {
        path,
        bits: port_kind.bits(),
        ty: port_kind.into(),
        id,
        slot,
    })
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]
pub enum LinkKind {
    Strong,
    Weak,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Hash)]

pub struct Link {
    pub from: PortId,
    pub to: PortId,
    pub kind: LinkKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CanonicalPathElement {
    /// An index into an array, e.g., `x[3]`
    Index(usize),
    /// A tuple index, e.g. `x.0`
    TupleIndex(usize),
    /// A struct field, e.g., `x.field_name`
    Field(Intern<String>),
    /// The enum discriminant, e.g., `x#`
    EnumDiscriminant,
    /// The enum payload (by name only)
    EnumPayload(Intern<String>),
    /// The value of a signal, e.g., `x@`
    SignalValue,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct CanonicalPath(Vec<CanonicalPathElement>);

impl FromIterator<CanonicalPathElement> for CanonicalPath {
    fn from_iter<T: IntoIterator<Item = CanonicalPathElement>>(iter: T) -> Self {
        CanonicalPath(iter.into_iter().collect())
    }
}

impl CanonicalPath {
    pub fn join(&self, other: &CanonicalPath) -> CanonicalPath {
        self.0
            .iter()
            .cloned()
            .chain(other.0.iter().cloned())
            .collect()
    }
    pub fn tuple_index(&self, index: usize) -> CanonicalPath {
        self.join(&CanonicalPath(vec![CanonicalPathElement::TupleIndex(
            index,
        )]))
    }
    pub fn field(&self, name: &str) -> CanonicalPath {
        self.join(&CanonicalPath(vec![CanonicalPathElement::Field(
            Intern::new(name.to_string()),
        )]))
    }
    pub fn signal_value(&self) -> CanonicalPath {
        self.join(&CanonicalPath(vec![CanonicalPathElement::SignalValue]))
    }
    pub fn index(&self, index: usize) -> CanonicalPath {
        self.join(&CanonicalPath(vec![CanonicalPathElement::Index(index)]))
    }
}

impl std::fmt::Debug for CanonicalPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for element in &self.0 {
            match element {
                CanonicalPathElement::Index(ndx) => write!(f, "[{ndx}]")?,
                CanonicalPathElement::TupleIndex(ndx) => write!(f, ".{ndx}")?,
                CanonicalPathElement::Field(name) => write!(f, ".{name}")?,
                CanonicalPathElement::EnumDiscriminant => write!(f, "#")?,
                CanonicalPathElement::EnumPayload(name) => write!(f, "#{name}")?,
                CanonicalPathElement::SignalValue => write!(f, "@")?,
            }
        }
        Ok(())
    }
}

fn canonicalize_path(mut kind: Kind, path: &Path) -> Result<CanonicalPath, RHDLError> {
    let base_kind = kind;
    let mut canonical_path = Vec::new();
    for element in path.iter() {
        match element {
            PathElement::Index(ndx) => {
                kind = sub_kind(kind, &Path::default().index(*ndx))?;
                canonical_path.push(CanonicalPathElement::Index(*ndx))
            }
            PathElement::TupleIndex(ndx) => {
                if kind.is_tuple() {
                    canonical_path.push(CanonicalPathElement::TupleIndex(*ndx))
                } else {
                    canonical_path.push(CanonicalPathElement::Field(ndx.to_string().into()))
                }
                kind = sub_kind(kind, &Path::default().tuple_index(*ndx))?;
            }
            PathElement::Field(name) => {
                kind = sub_kind(kind, &Path::default().field(name))?;
                canonical_path.push(CanonicalPathElement::Field(*name));
            }
            PathElement::EnumDiscriminant => {
                kind = sub_kind(kind, &Path::default().discriminant())?;
                canonical_path.push(CanonicalPathElement::EnumDiscriminant);
            }
            PathElement::EnumPayload(name) => {
                kind = sub_kind(kind, &Path::default().payload(name))?;
                canonical_path.push(CanonicalPathElement::EnumPayload(*name));
            }
            PathElement::EnumPayloadByValue(val) => {
                let variant = kind
                    .lookup_variant(*val)
                    .ok_or_else(|| {
                        rhdl_error(SchematicICE::UnableToCanonicalizePath {
                            path: path.clone(),
                            kind: base_kind,
                        })
                    })?
                    .name;
                kind = sub_kind(kind, &Path::default().payload(&variant))?;
                canonical_path.push(CanonicalPathElement::EnumPayload(variant));
            }
            PathElement::DynamicIndex(_) => {
                return Err(rhdl_error(SchematicICE::UnableToCanonicalizePath {
                    path: path.clone(),
                    kind: base_kind,
                }));
            }
            PathElement::SignalValue => {
                kind = sub_kind(kind, &Path::default().signal_value())?;
                canonical_path.push(CanonicalPathElement::SignalValue);
            }
        }
    }
    Ok(CanonicalPath(canonical_path))
}
