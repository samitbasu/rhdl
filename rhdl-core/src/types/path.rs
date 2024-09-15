use miette::Diagnostic;
use std::iter::once;
use std::ops::Range;
use thiserror::Error;

use crate::error::rhdl_error;
use crate::error::RHDLError;
use crate::rhif::spec::Member;
use crate::rhif::spec::Slot;
use crate::DiscriminantAlignment;
use crate::Kind;

#[derive(Error, Debug, Diagnostic)]
pub enum PathError {
    #[error("Path {prefix:?} is not a prefix of {path:?}")]
    NotAPrefix { prefix: Path, path: Path },
    #[error("Dynamic index {element:?} on non-array type {kind:?}")]
    DynamicIndexOnNonArray { element: PathElement, kind: Kind },
    #[error("Signal value not valid for non-signal type {kind:?}")]
    SignalValueOnNonSignal { kind: Kind },
    #[error("Tuple index {ndx} out of bounds for {kind:?}")]
    TupleIndexOutOfBounds { ndx: usize, kind: Kind },
    #[error("Struct index {ndx} out of bounds for {kind:?}")]
    StructIndexOutOfBounds { ndx: usize, kind: Kind },
    #[error("Tuple indexing not allowed on this type {kind:?}")]
    TupleIndexingNotAllowed { kind: Kind },
    #[error("Array index {ndx} out of bounds for {kind:?}")]
    ArrayIndexOutOfBounds { ndx: usize, kind: Kind },
    #[error("Indexing not allowed on this type {kind:?}")]
    IndexingNotAllowed { kind: Kind },
    #[error("Field {field} not found in {kind:?}")]
    FieldNotFound { field: String, kind: Kind },
    #[error("Field indexing not allowed on this type {kind:?}")]
    FieldIndexingNotAllowed { kind: Kind },
    #[error("Enum variant {name} payload not found for {kind:?}")]
    EnumPayloadNotFound { name: String, kind: Kind },
    #[error("Enum payload not valid for non-enum type {kind:?}")]
    EnumPayloadNotValid { kind: Kind },
    #[error("Enum payload not found for discriminant {disc} in {kind:?}")]
    EnumPayloadByValueNotFound { disc: i64, kind: Kind },
    #[error("Enum payload not valid for non-enum type {kind:?}")]
    EnumPayloadByValueNotValid { kind: Kind },
    #[error("Dynamic indices must be resolved {path:?} before calling bit_range")]
    DynamicIndicesNotResolved { path: Path },
}

type Result<T> = std::result::Result<T, RHDLError>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathElement {
    Index(usize),
    TupleIndex(usize),
    Field(String),
    EnumDiscriminant,
    EnumPayload(String),
    EnumPayloadByValue(i64),
    DynamicIndex(Slot),
    SignalValue,
}

#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct Path {
    pub elements: Vec<PathElement>,
}

impl FromIterator<PathElement> for Path {
    fn from_iter<T: IntoIterator<Item = PathElement>>(iter: T) -> Self {
        Path {
            elements: iter.into_iter().collect(),
        }
    }
}

impl std::fmt::Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for e in &self.elements {
            match e {
                PathElement::Index(i) => write!(f, "[{}]", i)?,
                PathElement::TupleIndex(i) => write!(f, ".{}", i)?,
                PathElement::Field(s) => write!(f, ".{}", s)?,
                PathElement::EnumDiscriminant => write!(f, "#")?,
                PathElement::EnumPayload(s) => write!(f, "#{}", s)?,
                PathElement::EnumPayloadByValue(v) => write!(f, "#{}", v)?,
                PathElement::DynamicIndex(slot) => write!(f, "[[{:?}]]", slot)?,
                PathElement::SignalValue => write!(f, "@")?,
            }
        }
        Ok(())
    }
}

impl Path {
    pub fn iter(&self) -> impl Iterator<Item = &PathElement> {
        self.elements.iter()
    }
    pub fn elements(self) -> impl Iterator<Item = PathElement> {
        self.elements.into_iter()
    }
    pub fn len(&self) -> usize {
        self.elements.len()
    }
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
    pub fn tuple_index(mut self, ndx: usize) -> Self {
        self.elements.push(PathElement::TupleIndex(ndx));
        self
    }
    pub fn field(mut self, field: &str) -> Self {
        self.elements.push(PathElement::Field(field.to_string()));
        self
    }
    pub fn signal_value(mut self) -> Self {
        self.elements.push(PathElement::SignalValue);
        self
    }
    pub fn member(mut self, member: &Member) -> Self {
        match member {
            Member::Named(name) => self.elements.push(PathElement::Field(name.to_owned())),
            Member::Unnamed(ndx) => self.elements.push(PathElement::TupleIndex(*ndx as usize)),
        }
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
        self.elements.is_empty()
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
    pub fn remap_slots<F: FnMut(Slot) -> Slot>(self, mut f: F) -> Path {
        Path {
            elements: self
                .elements
                .into_iter()
                .map(|e| match e {
                    PathElement::DynamicIndex(slot) => PathElement::DynamicIndex(f(slot)),
                    _ => e,
                })
                .collect(),
        }
    }
    pub fn is_prefix_of(&self, other: &Path) -> bool {
        self.elements.len() <= other.elements.len()
            && self
                .elements
                .iter()
                .zip(other.elements.iter())
                .all(|(a, b)| a == b)
    }
    pub fn strip_prefix(&self, prefix: &Path) -> Result<Path> {
        if !prefix.is_prefix_of(self) {
            return Err(rhdl_error(PathError::NotAPrefix {
                prefix: prefix.clone(),
                path: self.clone(),
            }));
        }
        Ok(Path {
            elements: self.elements[prefix.elements.len()..].to_vec(),
        })
    }

    pub fn is_magic_val_path(&self) -> bool {
        self.elements.len() == 1 && (self.elements[0] == PathElement::Field("#val".to_string()))
    }
    // Replace all dynamic indices such as `x[[a]]` with
    // simple indices `x[0]`.  Used to calculate the offset
    // of a dynamic indexing expression.
    pub fn zero_out_dynamic_indices(&self) -> Path {
        Path {
            elements: self
                .elements
                .iter()
                .map(|e| match e {
                    PathElement::DynamicIndex(_) => PathElement::Index(0),
                    _ => e.clone(),
                })
                .collect(),
        }
    }
    // Stride path - zero out all dynamic indices except the one
    // with the given slot.  In that case, use an index of 1.
    // This is equivalent to, um, differentiating the bit-range with
    // respect to the given slot.
    pub fn stride_path(&self, slot: Slot) -> Path {
        Path {
            elements: self
                .elements
                .iter()
                .map(|e| match e {
                    PathElement::DynamicIndex(s) if s == &slot => PathElement::Index(1),
                    PathElement::DynamicIndex(_) => PathElement::Index(0),
                    _ => e.clone(),
                })
                .collect(),
        }
    }
}

impl From<Member> for Path {
    fn from(member: Member) -> Self {
        match member {
            Member::Named(name) => Path {
                elements: vec![PathElement::Field(name.to_owned())],
            },
            Member::Unnamed(ndx) => Path {
                elements: vec![PathElement::TupleIndex(ndx as usize)],
            },
        }
    }
}

// Given a path and a kind, generate all leaf paths starting
// at the given path - these are paths that terminate in
// non-composite elements of a data structure.
pub fn leaf_paths(kind: &Kind, base: Path) -> Vec<Path> {
    match kind {
        Kind::Array(array) => (0..array.size)
            .flat_map(|i| leaf_paths(&array.base, base.clone().index(i)))
            .collect(),
        Kind::Tuple(tuple) => tuple
            .elements
            .iter()
            .enumerate()
            .flat_map(|(i, k)| leaf_paths(k, base.clone().index(i)))
            .collect(),
        Kind::Struct(structure) => structure
            .fields
            .iter()
            .flat_map(|field| leaf_paths(&field.kind, base.clone().field(&field.name)))
            .collect(),
        Kind::Signal(root, _) => leaf_paths(root, base.clone())
            .into_iter()
            .map(|path| path.signal_value())
            .collect(),
        Kind::Enum(enumeration) => enumeration
            .variants
            .iter()
            .flat_map(|variant| {
                leaf_paths(
                    &variant.kind,
                    base.clone().payload_by_value(variant.discriminant),
                )
            })
            .chain(once(base.clone().discriminant()))
            .collect(),
        Kind::Bits(_) | Kind::Signed(_) | Kind::Empty => vec![base.clone()],
    }
}

// Given a path and a kind, computes all possible paths that can be
// generated from the base path using legal values for the dynamic
// indices.
pub fn path_star(kind: &Kind, path: &Path) -> Result<Vec<Path>> {
    eprintln!("path star called with kind {:?} and path {:?}", kind, path);
    if !path.any_dynamic() {
        return Ok(vec![path.clone()]);
    }
    if let Some(element) = path.elements.first() {
        match element {
            PathElement::DynamicIndex(_) => {
                let Kind::Array(array) = kind else {
                    return Err(rhdl_error(PathError::DynamicIndexOnNonArray {
                        element: element.clone(),
                        kind: kind.clone(),
                    }));
                };
                let mut paths = Vec::new();
                for i in 0..array.size {
                    let mut path = path.clone();
                    path.elements[0] = PathElement::Index(i);
                    paths.extend(path_star(kind, &path)?);
                }
                return Ok(paths);
            }
            p => {
                let prefix_path = Path {
                    elements: vec![p.clone()],
                };
                let prefix_kind = sub_kind(kind.clone(), &prefix_path)?;
                let suffix_path = path.strip_prefix(&prefix_path)?;
                let suffix_star = path_star(&prefix_kind, &suffix_path)?;
                return Ok(suffix_star
                    .into_iter()
                    .map(|suffix| prefix_path.clone().join(&suffix))
                    .collect());
            }
        }
    }
    Ok(vec![path.clone()])
}

pub fn sub_kind(kind: Kind, path: &Path) -> Result<Kind> {
    bit_range(kind, path).map(|(_, kind)| kind)
}

// Given a Kind and a Vec<Path>, compute the bit offsets of
// the endpoint of the path within the original data structure.
pub fn bit_range(kind: Kind, path: &Path) -> Result<(Range<usize>, Kind)> {
    let mut range = 0..kind.bits();
    let mut kind = kind;
    for p in &path.elements {
        match p {
            PathElement::SignalValue => {
                if let Kind::Signal(root, _) = kind {
                    kind = *root.clone();
                } else {
                    return Err(rhdl_error(PathError::SignalValueOnNonSignal {
                        kind: kind.clone(),
                    }));
                }
            }
            PathElement::TupleIndex(i) => match &kind {
                Kind::Tuple(tuple) => {
                    if i >= &tuple.elements.len() {
                        return Err(rhdl_error(PathError::TupleIndexOutOfBounds {
                            ndx: *i,
                            kind: kind.clone(),
                        }));
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
                        return Err(rhdl_error(PathError::StructIndexOutOfBounds {
                            ndx: *i,
                            kind: kind.clone(),
                        }));
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
                _ => {
                    return Err(rhdl_error(PathError::TupleIndexingNotAllowed {
                        kind: kind.clone(),
                    }))
                }
            },
            PathElement::Index(i) => match &kind {
                Kind::Array(array) => {
                    let element_size = array.base.bits();
                    if i >= &array.size {
                        return Err(rhdl_error(PathError::ArrayIndexOutOfBounds {
                            ndx: *i,
                            kind: kind.clone(),
                        }));
                    }
                    range = range.start + i * element_size..range.start + (i + 1) * element_size;
                    kind = *array.base.clone();
                }
                Kind::Struct(structure) => {
                    if i >= &structure.fields.len() {
                        return Err(rhdl_error(PathError::StructIndexOutOfBounds {
                            ndx: *i,
                            kind: kind.clone(),
                        }));
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
                Kind::Signal(root, _) if matches!(root.as_ref(), Kind::Array(_)) => {
                    let Kind::Array(array) = root.as_ref() else {
                        return Err(rhdl_error(PathError::IndexingNotAllowed {
                            kind: *root.clone(),
                        }));
                    };
                    let element_size = array.base.bits();
                    if i >= &array.size {
                        return Err(rhdl_error(PathError::ArrayIndexOutOfBounds {
                            ndx: *i,
                            kind: kind.clone(),
                        }));
                    }
                    range = range.start + i * element_size..range.start + (i + 1) * element_size;
                    kind = *array.base.clone();
                }
                _ => {
                    return Err(rhdl_error(PathError::IndexingNotAllowed {
                        kind: kind.clone(),
                    }))
                }
            },
            PathElement::Field(field) => match &kind {
                Kind::Struct(structure) => {
                    if !structure.fields.iter().any(|f| &f.name == field) {
                        return Err(rhdl_error(PathError::FieldNotFound {
                            field: field.clone(),
                            kind: kind.clone(),
                        }));
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
                _ => {
                    return Err(rhdl_error(PathError::FieldIndexingNotAllowed {
                        kind: kind.clone(),
                    }))
                }
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
                _ => {
                    // For non-enum types, the discriminant is the value itself
                }
            },
            PathElement::EnumPayload(name) => match &kind {
                Kind::Enum(enumerate) => {
                    let field = enumerate
                        .variants
                        .iter()
                        .find(|f| &f.name == name)
                        .ok_or_else(|| {
                            rhdl_error(PathError::EnumPayloadNotFound {
                                name: name.clone(),
                                kind: kind.clone(),
                            })
                        })?
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
                _ => {
                    return Err(rhdl_error(PathError::EnumPayloadNotValid {
                        kind: kind.clone(),
                    }))
                }
            },
            PathElement::EnumPayloadByValue(disc) => match &kind {
                Kind::Enum(enumerate) => {
                    let field = enumerate
                        .variants
                        .iter()
                        .find(|f| f.discriminant == *disc)
                        .ok_or_else(|| {
                            rhdl_error(PathError::EnumPayloadByValueNotFound {
                                disc: *disc,
                                kind: kind.clone(),
                            })
                        })?
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
                _ => {
                    return Err(rhdl_error(PathError::EnumPayloadByValueNotValid {
                        kind: kind.clone(),
                    }))
                }
            },
            PathElement::DynamicIndex(_slot) => {
                return Err(rhdl_error(PathError::DynamicIndicesNotResolved {
                    path: path.clone(),
                }))
            }
        }
    }
    Ok((range, kind))
}

#[cfg(test)]
mod tests {
    use crate::{
        rhif::spec::{RegisterId, Slot},
        types::{kind::DiscriminantLayout, path::path_star},
        Kind,
    };

    use super::{leaf_paths, Path};

    #[test]
    fn test_leaf_path() {
        let base_struct = Kind::make_struct(
            "base",
            vec![
                Kind::make_field("a", Kind::make_bits(8)),
                Kind::make_field("b", Kind::make_array(Kind::make_bits(8), 3)),
            ],
        );
        // Create a path with a struct, containing and array of structs
        let lev2 = Kind::make_struct(
            "foo",
            vec![
                Kind::make_field("c", base_struct.clone()),
                Kind::make_field("d", Kind::make_array(base_struct.clone(), 4)),
            ],
        );
        let kind = Kind::make_enum(
            "bar",
            vec![
                Kind::make_variant("a", Kind::make_bits(8), 0),
                Kind::make_variant("b", lev2.clone(), 1),
            ],
            DiscriminantLayout {
                width: 8,
                alignment: crate::DiscriminantAlignment::Lsb,
                ty: crate::DiscriminantType::Unsigned,
            },
        );
        let mut bit_mask = vec![false; kind.bits()];
        for path in leaf_paths(&kind, Path::default()) {
            let (range, _) = super::bit_range(kind.clone(), &path).unwrap();
            for i in range {
                bit_mask[i] = true;
            }
        }
        assert!(bit_mask.iter().all(|b| *b));
    }

    #[test]
    fn test_path_star() {
        let base_struct = Kind::make_struct(
            "base",
            vec![
                Kind::make_field("a", Kind::make_bits(8)),
                Kind::make_field("b", Kind::make_array(Kind::make_bits(8), 3)),
            ],
        );
        // Create a path with a struct, containing and array of structs
        let kind = Kind::make_struct(
            "foo",
            vec![
                Kind::make_field("c", base_struct.clone()),
                Kind::make_field("d", Kind::make_array(base_struct.clone(), 4)),
            ],
        );
        let path1 = Path::default().field("c").field("a");
        assert_eq!(path_star(&kind, &path1).unwrap().len(), 1);
        let path1 = Path::default().field("c").field("b");
        assert_eq!(path_star(&kind, &path1).unwrap().len(), 1);
        let path1 = Path::default().field("c").field("b").index(0);
        assert_eq!(path_star(&kind, &path1).unwrap().len(), 1);
        let path1 = Path::default()
            .field("c")
            .field("b")
            .dynamic(Slot::Register(RegisterId(0)));
        let path1_star = path_star(&kind, &path1).unwrap();
        assert_eq!(path1_star.len(), 3);
        for path in path1_star {
            assert_eq!(path.elements.len(), 3);
            assert!(!path.any_dynamic());
            eprintln!("{:?}", path);
        }
        let path2 = Path::default()
            .field("d")
            .dynamic(Slot::Register(RegisterId(0)))
            .field("b");
        let path2_star = path_star(&kind, &path2).unwrap();
        assert_eq!(path2_star.len(), 4);
        for path in path2_star {
            assert_eq!(path.elements.len(), 3);
            assert!(!path.any_dynamic());
            eprintln!("{:?}", path);
        }
        let path3 = Path::default()
            .field("d")
            .dynamic(Slot::Register(RegisterId(0)))
            .field("b")
            .dynamic(Slot::Register(RegisterId(1)));
        let path3_star = path_star(&kind, &path3).unwrap();
        assert_eq!(path3_star.len(), 12);
        for path in path3_star {
            assert_eq!(path.elements.len(), 4);
            assert!(!path.any_dynamic());
            eprintln!("{:?}", path);
        }
    }
}
