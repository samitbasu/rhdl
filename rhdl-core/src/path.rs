use std::ops::Range;

use anyhow::bail;
use anyhow::Result;

use crate::ast::Member;
use crate::rhif::Slot;
use crate::DiscriminantAlignment;
use crate::Kind;

#[derive(Debug, Clone, PartialEq)]
pub enum PathElement {
    All,
    Index(usize),
    Field(String),
    EnumDiscriminant,
    EnumPayload(String),
    EnumPayloadByValue(i64),
    DynamicIndex(Slot),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Path {
    pub elements: Vec<PathElement>,
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for e in &self.elements {
            match e {
                PathElement::All => write!(f, "")?,
                PathElement::Index(i) => write!(f, "[{}]", i)?,
                PathElement::Field(s) => write!(f, ".{}", s)?,
                PathElement::EnumDiscriminant => write!(f, "#")?,
                PathElement::EnumPayload(s) => write!(f, "#{}", s)?,
                PathElement::EnumPayloadByValue(v) => write!(f, "#{}", v)?,
                PathElement::DynamicIndex(slot) => write!(f, "[[{}]]", slot)?,
            }
        }
        Ok(())
    }
}

impl Path {
    pub fn dynamic_slots(&self) -> impl Iterator<Item = &Slot> {
        self.elements.iter().filter_map(|e| match e {
            PathElement::DynamicIndex(slot) => Some(slot),
            _ => None,
        })
    }
    pub fn index(mut self, index: usize) -> Self {
        self.elements.push(PathElement::Index(index));
        self
    }
    pub fn field(mut self, field: &str) -> Self {
        self.elements.push(PathElement::Field(field.to_string()));
        self
    }
    pub fn discriminant(mut self) -> Self {
        self.elements.push(PathElement::EnumDiscriminant);
        self
    }
    pub fn dynamic(mut self, slot: Slot) -> Self {
        self.elements.push(PathElement::DynamicIndex(slot));
        self
    }
    pub fn payload(mut self, name: &str) -> Self {
        self.elements
            .push(PathElement::EnumPayload(name.to_string()));
        self
    }
    pub fn join(mut self, other: &Path) -> Self {
        self.elements.extend(other.elements.clone());
        self
    }
    pub fn is_empty(&self) -> bool {
        self.elements.iter().all(|e| matches!(e, PathElement::All))
    }
    pub fn payload_by_value(mut self, discriminant: i64) -> Self {
        self.elements
            .push(PathElement::EnumPayloadByValue(discriminant));
        self
    }
    pub fn any_dynamic(&self) -> bool {
        self.elements
            .iter()
            .any(|e| matches!(e, PathElement::DynamicIndex(_)))
    }
}

impl From<Member> for Path {
    fn from(member: Member) -> Self {
        match member {
            Member::Named(name) => Path {
                elements: vec![PathElement::Field(name)],
            },
            Member::Unnamed(ndx) => Path {
                elements: vec![PathElement::Index(ndx as usize)],
            },
        }
    }
}

// Given a Kind and a Vec<Path>, compute the bit offsets of
// the endpoint of the path within the original data structure.
pub fn bit_range(kind: Kind, path: &Path) -> Result<(Range<usize>, Kind)> {
    let mut range = 0..kind.bits();
    let mut kind = kind;
    for p in &path.elements {
        match p {
            PathElement::All => (),
            PathElement::Index(i) => match &kind {
                Kind::Array(array) => {
                    let element_size = array.base.bits();
                    if i >= &array.size {
                        bail!("Array index out of bounds")
                    }
                    range = range.start + i * element_size..range.start + (i + 1) * element_size;
                    kind = *array.base.clone();
                }
                Kind::Tuple(tuple) => {
                    if i >= &tuple.elements.len() {
                        bail!("Tuple index out of bounds")
                    }
                    let offset = tuple.elements[0..*i]
                        .iter()
                        .map(|e| e.bits())
                        .sum::<usize>();
                    let size = tuple.elements[*i].bits();
                    range = range.start + offset..range.start + offset + size;
                    kind = tuple.elements[*i].clone();
                }
                Kind::Struct(structure) => {
                    if i >= &structure.fields.len() {
                        bail!("Struct index out of bounds")
                    }
                    let offset = structure
                        .fields
                        .iter()
                        .take(*i)
                        .map(|f| f.kind.bits())
                        .sum::<usize>();
                    let size = structure.fields[*i].kind.bits();
                    range = range.start + offset..range.start + offset + size;
                    kind = structure.fields[*i].kind.clone();
                }
                _ => bail!("Indexing non-indexable type {kind}"),
            },
            PathElement::Field(field) => match &kind {
                Kind::Struct(structure) => {
                    if !structure.fields.iter().any(|f| &f.name == field) {
                        bail!("Field not found")
                    }
                    let offset = structure
                        .fields
                        .iter()
                        .take_while(|f| &f.name != field)
                        .map(|f| f.kind.bits())
                        .sum::<usize>();
                    let field = &structure
                        .fields
                        .iter()
                        .find(|f| &f.name == field)
                        .unwrap()
                        .kind;
                    let size = field.bits();
                    range = range.start + offset..range.start + offset + size;
                    kind = field.clone();
                }
                _ => bail!("Field indexing not allowed on this type {kind}"),
            },
            PathElement::EnumDiscriminant => match &kind {
                Kind::Enum(enumerate) => {
                    range = match enumerate.discriminant_layout.alignment {
                        DiscriminantAlignment::Lsb => {
                            range.start..range.start + enumerate.discriminant_layout.width
                        }
                        DiscriminantAlignment::Msb => {
                            range.end - enumerate.discriminant_layout.width..range.end
                        }
                    };
                    kind = if enumerate.discriminant_layout.ty == crate::DiscriminantType::Signed {
                        Kind::make_signed(enumerate.discriminant_layout.width)
                    } else {
                        Kind::make_bits(enumerate.discriminant_layout.width)
                    };
                }
                _ => bail!("Enum discriminant not valid for non-enum types"),
            },
            PathElement::EnumPayload(name) => match &kind {
                Kind::Enum(enumerate) => {
                    let field = enumerate
                        .variants
                        .iter()
                        .find(|f| &f.name == name)
                        .ok_or_else(|| anyhow::anyhow!("Enum payload not found"))?
                        .kind
                        .clone();
                    range = match enumerate.discriminant_layout.alignment {
                        DiscriminantAlignment::Lsb => {
                            range.start + enumerate.discriminant_layout.width
                                ..range.start + enumerate.discriminant_layout.width + field.bits()
                        }
                        DiscriminantAlignment::Msb => range.start..range.start + field.bits(),
                    };
                    kind = field;
                }
                _ => bail!("Enum payload not valid for non-enum types"),
            },
            PathElement::EnumPayloadByValue(disc) => match &kind {
                Kind::Enum(enumerate) => {
                    let field = enumerate
                        .variants
                        .iter()
                        .find(|f| f.discriminant == *disc)
                        .ok_or_else(|| anyhow::anyhow!("Enum payload not found"))?
                        .kind
                        .clone();
                    range = match enumerate.discriminant_layout.alignment {
                        DiscriminantAlignment::Lsb => {
                            range.start + enumerate.discriminant_layout.width
                                ..range.start + enumerate.discriminant_layout.width + field.bits()
                        }
                        DiscriminantAlignment::Msb => range.start..range.start + field.bits(),
                    };
                    kind = field;
                }
                _ => bail!("Enum payload not valid for non-enum types"),
            },
            PathElement::DynamicIndex(_slot) => {
                bail!("Dynamic indices must be resolved before calling bit_range")
            }
        }
    }
    Ok((range, kind))
}
