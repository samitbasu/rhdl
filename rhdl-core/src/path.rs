use std::ops::Range;

use anyhow::bail;
use anyhow::Result;

use crate::Kind;

#[derive(Debug, Clone, PartialEq)]
pub enum Path {
    All,
    Index(usize),
    Field(&'static str),
    EnumDiscriminant,
    EnumPayload(&'static str),
}

// Given a Kind and a Vec<Path>, compute the bit offsets of
// the endpoint of the path within the original data structure.
pub fn bit_range(kind: Kind, path: &[Path]) -> Result<(Range<usize>, Kind)> {
    let mut range = 0..kind.bits();
    let mut kind = kind;
    for p in path {
        match p {
            Path::All => (),
            Path::Index(i) => {
                match &kind {
                    Kind::Array(array) => {
                        let element_size = array.base.bits();
                        if i >= &array.size {
                            bail!("Array index out of bounds")
                        }
                        range =
                            range.start + i * element_size..range.start + (i + 1) * element_size;
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
                    _ => bail!("Indexing non-indexable type"),
                }
                range = range.start + i..range.start + i + 1;
            }
            Path::Field(field) => match &kind {
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
            Path::EnumDiscriminant => match &kind {
                Kind::Enum(enumerate) => {
                    range = range.start..range.start + enumerate.discriminant_width;
                    kind = Kind::Bits(enumerate.discriminant_width);
                }
                _ => bail!("Enum discriminant not valid for non-enum types"),
            },
            Path::EnumPayload(name) => match &kind {
                Kind::Enum(enumerate) => {
                    let field = enumerate
                        .variants
                        .iter()
                        .find(|f| &f.name == name)
                        .ok_or_else(|| anyhow::anyhow!("Enum payload not found"))?
                        .kind
                        .clone();
                    range = range.start + enumerate.discriminant_width
                        ..range.start + enumerate.discriminant_width + field.bits();
                    kind = field;
                }
                _ => bail!("Enum payload not valid for non-enum types"),
            },
        }
    }
    Ok((range, kind))
}
