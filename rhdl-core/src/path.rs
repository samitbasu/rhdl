use std::ops::Range;

use anyhow::bail;
use anyhow::Result;

use crate::ast::Member;
use crate::DiscriminantAlignment;
use crate::Kind;

#[derive(Debug, Clone, PartialEq)]
pub enum PathElement {
    All,
    Index(usize),
    Field(String),
    EnumDiscriminant,
    EnumPayload(String),
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
            }
        }
        Ok(())
    }
}

impl Path {
    pub fn index(self, index: usize) -> Self {
        let mut elements = self.elements;
        elements.push(PathElement::Index(index));
        Path { elements }
    }
    pub fn field(self, field: &str) -> Self {
        let mut elements = self.elements;
        elements.push(PathElement::Field(field.to_string()));
        Path { elements }
    }
    pub fn discriminant(self) -> Self {
        let mut elements = self.elements;
        elements.push(PathElement::EnumDiscriminant);
        Path { elements }
    }
    pub fn payload(self, name: &str) -> Self {
        let mut elements = self.elements;
        elements.push(PathElement::EnumPayload(name.to_string()));
        Path { elements }
    }
    pub fn join(self, other: &Path) -> Self {
        let mut elements = self.elements;
        elements.extend(other.elements.clone());
        Path { elements }
    }
    pub fn is_empty(&self) -> bool {
        self.elements.iter().all(|e| matches!(e, PathElement::All))
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
                _ => bail!("Indexing non-indexable type"),
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
                _ => bail!("Field indexing not allowed on this type"),
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
                    dbg!(&range);
                    kind = field;
                }
                _ => bail!("Enum payload not valid for non-enum types"),
            },
        }
    }
    Ok((range, kind))
}
