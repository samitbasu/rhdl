//! A path representation for indexing into [Digital](crate::types::digital::Digital) types.
use internment::Intern;
use miette::Diagnostic;
use rhdl_trace_type::TraceType;
use std::iter::once;
use std::ops::Range;
use thiserror::Error;

use crate::DiscriminantAlignment;
use crate::Kind;
use crate::error::RHDLError;
use crate::error::rhdl_error;
use crate::rhif::spec::Member;
use crate::rhif::spec::Slot;

#[allow(missing_docs)]
#[derive(Error, Debug, Diagnostic)]
pub enum PathError {
    #[error("Path {prefix:?} is not a prefix of {path:?}")]
    NotAPrefix { prefix: Path, path: Path },
    #[error("Dynamic index {element:?} on non-array type {kind:?}")]
    DynamicIndexOnNonArray { element: PathElement, kind: Kind },
    #[error("Signal value not valid for non-signal type {kind:?}")]
    SignalValueOnNonSignal { kind: Kind },
    #[error("Signal value not valid for non-signal type {trace:?}")]
    SignalValueOnNonSignalTrace { trace: TraceType },
    #[error("Tuple index {ndx} out of bounds for {kind:?}")]
    TupleIndexOutOfBounds { ndx: usize, kind: Kind },
    #[error("Tuple index {ndx} out of bounds for {trace:?}")]
    TupleIndexOutOfBoundsTrace { ndx: usize, trace: TraceType },
    #[error("Struct index {ndx} out of bounds for {kind:?}")]
    StructIndexOutOfBounds { ndx: usize, kind: Kind },
    #[error("Tuple indexing not allowed on this type {kind:?}")]
    TupleIndexingNotAllowed { kind: Kind },
    #[error("Tuple indexing not allowed on this type {trace:?}")]
    TupleIndexingNotAllowedTrace { trace: TraceType },
    #[error("Array index {ndx} out of bounds for {kind:?}")]
    ArrayIndexOutOfBounds { ndx: usize, kind: Kind },
    #[error("Array index {ndx} out of bounds for {trace:?}")]
    ArrayIndexOutOfBoundsTrace { ndx: usize, trace: TraceType },
    #[error("Indexing not allowed on this type {kind:?}")]
    IndexingNotAllowed { kind: Kind },
    #[error("Array indexing not allowed on this type {trace:?}")]
    IndexingNotAllowedTrace { trace: TraceType },
    #[error("Field {field} not found in {kind:?}")]
    FieldNotFound { field: Intern<String>, kind: Kind },
    #[error("Field {field} not found in {trace:?}")]
    FieldNotFoundTrace {
        field: Intern<String>,
        trace: TraceType,
    },
    #[error("Field indexing not allowed on this type {kind:?}")]
    FieldIndexingNotAllowed { kind: Kind },
    #[error("Field indexing not allowed on this type {trace:?}")]
    FieldIndexingNotAllowedTrace { trace: TraceType },
    #[error("Enum variant {name} payload not found for {kind:?}")]
    EnumPayloadNotFound { name: Intern<String>, kind: Kind },
    #[error("Enum payload not valid for non-enum type {kind:?}")]
    EnumPayloadNotValid { kind: Kind },
    #[error("Enum payload not found for discriminant {disc} in {kind:?}")]
    EnumPayloadByValueNotFound { disc: i64, kind: Kind },
    #[error("Enum payload not valid for non-enum type {kind:?}")]
    EnumPayloadByValueNotValid { kind: Kind },
    #[error("Dynamic indices must be resolved {path:?} before calling bit_range")]
    DynamicIndicesNotResolved { path: Path },
    #[error("Unsupported path type {path:?} for trace {trace:?}")]
    UnsupportedPathTypeForTrace { path: Path, trace: TraceType },
}

type Result<T> = std::result::Result<T, RHDLError>;

/// An element of a [Path](crate::types::path::Path).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathElement {
    /// An index into an array, e.g. `x[3]`
    Index(usize),
    /// A tuple index, e.g. `x.0`
    TupleIndex(usize),
    /// A struct field, e.g. `x.field_name`
    Field(Intern<String>),
    /// The enum discriminant, e.g. `x#`
    EnumDiscriminant,
    /// The enum payload, e.g. `x#VariantName`
    EnumPayload(Intern<String>),
    /// The enum payload by discriminant value, e.g. `x#0`
    EnumPayloadByValue(i64),
    /// A dynamic index, e.g. `x[y]`
    DynamicIndex(Slot),
    /// The value of a Signal, e.g. `x@`
    SignalValue,
}

/// A path for indexing into [Digital](crate::types::digital::Digital) types.
#[derive(Clone, PartialEq, Hash, Default)]
pub struct Path {
    elements: Vec<PathElement>,
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
        for e in self.elements.iter() {
            match e {
                PathElement::Index(i) => write!(f, "[{i}]")?,
                PathElement::TupleIndex(i) => write!(f, ".{i}")?,
                PathElement::Field(s) => write!(f, ".{s}")?,
                PathElement::EnumDiscriminant => write!(f, "#")?,
                PathElement::EnumPayload(s) => write!(f, "#{s}")?,
                PathElement::EnumPayloadByValue(v) => write!(f, "#{v}")?,
                PathElement::DynamicIndex(slot) => write!(f, "[[{slot:?}]]")?,
                PathElement::SignalValue => write!(f, "@")?,
            }
        }
        Ok(())
    }
}

impl Path {
    /// Get an iterator over the path elements.
    pub fn iter(&self) -> impl Iterator<Item = &PathElement> {
        self.elements.iter()
    }
    /// Get an iterator over the path elements by value.
    pub fn elements(self) -> impl Iterator<Item = PathElement> {
        self.elements.into_iter()
    }
    /// Get the number of elements in the path.
    pub fn len(&self) -> usize {
        self.elements.len()
    }
    /// Get an iterator over the dynamic slots in the path.
    pub fn dynamic_slots(&self) -> impl Iterator<Item = &Slot> {
        self.elements.iter().filter_map(|e| match e {
            PathElement::DynamicIndex(slot) => Some(slot),
            _ => None,
        })
    }
    /// Get a mutable iterator over the dynamic slots in the path.
    pub fn dynamic_slots_mut(&mut self) -> impl Iterator<Item = &mut Slot> {
        self.elements.iter_mut().filter_map(|e| match e {
            PathElement::DynamicIndex(slot) => Some(slot),
            _ => None,
        })
    }
    /// Add an index element to the path.
    pub fn index(mut self, index: usize) -> Self {
        self.elements.push(PathElement::Index(index));
        self
    }
    /// Add a tuple index element to the path.
    pub fn tuple_index(mut self, ndx: usize) -> Self {
        self.elements.push(PathElement::TupleIndex(ndx));
        self
    }
    /// Add a field element to the path.
    pub fn field(mut self, field: &str) -> Self {
        self.elements
            .push(PathElement::Field(field.to_string().into()));
        self
    }
    /// Add a signal value element to the path.
    pub fn signal_value(mut self) -> Self {
        self.elements.push(PathElement::SignalValue);
        self
    }
    /// Add a member element to the path.
    pub fn member(mut self, member: &Member) -> Self {
        match member {
            Member::Named(name) => self.elements.push(PathElement::Field(name.to_owned())),
            Member::Unnamed(ndx) => self.elements.push(PathElement::TupleIndex(*ndx as usize)),
        }
        self
    }
    /// Add a discriminant element to the path.
    pub fn discriminant(mut self) -> Self {
        self.elements.push(PathElement::EnumDiscriminant);
        self
    }
    /// Add a dynamic index element to the path.
    pub fn dynamic(mut self, slot: Slot) -> Self {
        self.elements.push(PathElement::DynamicIndex(slot));
        self
    }
    /// Add a payload element to the path.
    pub fn payload(mut self, name: &str) -> Self {
        self.elements
            .push(PathElement::EnumPayload(name.to_string().into()));
        self
    }
    /// Add a payload by value element to the path.
    pub fn join(mut self, other: &Path) -> Self {
        self.elements.extend(other.elements.clone());
        self
    }
    /// Check if the path is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
    /// Add a payload by value element to the path.
    pub fn payload_by_value(mut self, discriminant: i64) -> Self {
        self.elements
            .push(PathElement::EnumPayloadByValue(discriminant));
        self
    }
    /// Check if the path contains any dynamic indices.
    pub fn any_dynamic(&self) -> bool {
        self.elements
            .iter()
            .any(|e| matches!(e, PathElement::DynamicIndex(_)))
    }
    /// Remap all dynamic indices using the given function.
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
    /// Check if the path is a prefix of another path.
    pub fn is_prefix_of(&self, other: &Path) -> bool {
        self.elements.len() <= other.elements.len()
            && self
                .elements
                .iter()
                .zip(other.elements.iter())
                .all(|(a, b)| a == b)
    }
    /// Strip the given prefix from the path.
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
    /// Check if the path is the magic `#val` path.
    pub fn is_magic_val_path(&self) -> bool {
        self.elements.len() == 1
            && (self.elements[0] == PathElement::Field("#val".to_string().into()))
    }
    /// Replace all dynamic indices such as `x[[a]]` with
    /// simple indices `x[0]`.  Used to calculate the offset
    /// of a dynamic indexing expression.
    pub fn zero_out_dynamic_indices(&self) -> Path {
        Path {
            elements: self
                .elements
                .iter()
                .map(|e| match e {
                    PathElement::DynamicIndex(_) => PathElement::Index(0),
                    _ => *e,
                })
                .collect(),
        }
    }
    /// Stride path - zero out all dynamic indices except the one
    /// with the given slot.  In that case, use an index of 1.
    /// This is equivalent to, um, differentiating the bit-range with
    /// respect to the given slot.
    pub fn stride_path(&self, slot: Slot) -> Path {
        Path {
            elements: self
                .elements
                .iter()
                .map(|e| match e {
                    PathElement::DynamicIndex(s) if s == &slot => PathElement::Index(1),
                    PathElement::DynamicIndex(_) => PathElement::Index(0),
                    _ => *e,
                })
                .collect(),
        }
    }
    /// Create a path with a single element.
    pub(crate) fn with_element(x: PathElement) -> Path {
        Path { elements: vec![x] }
    }
    /// Pop the last element from the path.
    pub(crate) fn pop(&mut self) -> Option<PathElement> {
        self.elements.pop()
    }
    /// Push an element to the path.
    pub(crate) fn push(&mut self, element: PathElement) {
        self.elements.push(element);
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
pub(crate) fn leaf_paths(kind: &Kind, base: Path) -> Vec<Path> {
    match kind {
        Kind::Array(array) => (0..array.size)
            .flat_map(|i| leaf_paths(&array.base, base.clone().index(i)))
            .collect(),
        Kind::Tuple(tuple) => tuple
            .elements
            .iter()
            .enumerate()
            .flat_map(|(i, k)| leaf_paths(k, base.clone().tuple_index(i)))
            .collect(),
        Kind::Struct(structure) => structure
            .fields
            .iter()
            .flat_map(|field| leaf_paths(&field.kind, base.clone().field(&field.name)))
            .collect(),
        Kind::Signal(root, _) => leaf_paths(root, base.clone().signal_value()),
        Kind::Enum(enumeration) => enumeration
            .variants
            .iter()
            .flat_map(|variant| leaf_paths(&variant.kind, base.clone().payload(&variant.name)))
            .chain(once(base.clone().discriminant()))
            .collect(),
        Kind::Bits(_) | Kind::Signed(_) | Kind::Empty => vec![base.clone()],
    }
}

/// Given a [Kind] and a [Path], compute the [Kind] at the endpoint of the path.
pub fn sub_kind(kind: Kind, path: &Path) -> Result<Kind> {
    bit_range(kind, path).map(|(_, kind)| kind)
}

/// Given a [TraceType] and a [Path], compute the [TraceType] at the endpoint of the path.
pub fn sub_trace_type(trace: TraceType, path: &Path) -> Result<TraceType> {
    let mut trace = trace;
    for p in &path.elements {
        match p {
            PathElement::SignalValue => {
                if let TraceType::Signal(base, _) = trace {
                    trace = *base;
                } else {
                    return Err(rhdl_error(PathError::SignalValueOnNonSignalTrace { trace }));
                }
            }
            PathElement::TupleIndex(i) => match &trace {
                TraceType::Tuple(tuple) => {
                    if i >= &tuple.elements.len() {
                        return Err(rhdl_error(PathError::TupleIndexOutOfBoundsTrace {
                            ndx: *i,
                            trace,
                        }));
                    }
                    trace = tuple.elements[*i].clone();
                }
                _ => {
                    return Err(rhdl_error(PathError::TupleIndexingNotAllowedTrace {
                        trace,
                    }));
                }
            },
            PathElement::Index(i) => match &trace {
                TraceType::Array(array) => {
                    if i >= &array.size {
                        return Err(rhdl_error(PathError::ArrayIndexOutOfBoundsTrace {
                            ndx: *i,
                            trace,
                        }));
                    }
                    trace = *array.base.clone();
                }
                _ => return Err(rhdl_error(PathError::IndexingNotAllowedTrace { trace })),
            },
            PathElement::Field(field) => match &trace {
                TraceType::Struct(strukt) => {
                    if !strukt.fields.iter().any(|f| f.name == **field) {
                        return Err(rhdl_error(PathError::FieldNotFoundTrace {
                            field: *field,
                            trace,
                        }));
                    }
                    let field = &strukt.fields.iter().find(|f| f.name == **field).unwrap().ty;
                    trace = field.clone();
                }
                _ => {
                    return Err(rhdl_error(PathError::FieldIndexingNotAllowedTrace {
                        trace,
                    }));
                }
            },
            _ => {
                return Err(rhdl_error(PathError::UnsupportedPathTypeForTrace {
                    path: path.clone(),
                    trace,
                }));
            }
        }
    }
    Ok(trace)
}

/// Given a [Kind] and a [Path], compute the bit offsets of
/// the endpoint of the path within the original data structure,
/// as well as the [Kind] at that endpoint.
pub fn bit_range(kind: Kind, path: &Path) -> Result<(Range<usize>, Kind)> {
    let mut range = 0..kind.bits();
    let mut kind = kind;
    for p in &path.elements {
        match p {
            PathElement::SignalValue => {
                if let Kind::Signal(root, _) = kind {
                    kind = *root;
                } else {
                    return Err(rhdl_error(PathError::SignalValueOnNonSignal { kind }));
                }
            }
            PathElement::TupleIndex(i) => match &kind {
                Kind::Tuple(tuple) => {
                    if i >= &tuple.elements.len() {
                        return Err(rhdl_error(PathError::TupleIndexOutOfBounds {
                            ndx: *i,
                            kind,
                        }));
                    }
                    let offset = tuple.elements[0..*i]
                        .iter()
                        .map(|e| e.bits())
                        .sum::<usize>();
                    let size = tuple.elements[*i].bits();
                    range = range.start + offset..range.start + offset + size;
                    kind = tuple.elements[*i];
                }
                Kind::Struct(structure) => {
                    if i >= &structure.fields.len() {
                        return Err(rhdl_error(PathError::StructIndexOutOfBounds {
                            ndx: *i,
                            kind,
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
                    kind = structure.fields[*i].kind;
                }
                _ => return Err(rhdl_error(PathError::TupleIndexingNotAllowed { kind })),
            },
            PathElement::Index(i) => match &kind {
                Kind::Array(array) => {
                    let element_size = array.base.bits();
                    if i >= &array.size {
                        return Err(rhdl_error(PathError::ArrayIndexOutOfBounds {
                            ndx: *i,
                            kind,
                        }));
                    }
                    range = range.start + i * element_size..range.start + (i + 1) * element_size;
                    kind = *array.base;
                }
                Kind::Struct(structure) => {
                    if i >= &structure.fields.len() {
                        return Err(rhdl_error(PathError::StructIndexOutOfBounds {
                            ndx: *i,
                            kind,
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
                    kind = structure.fields[*i].kind;
                }
                Kind::Signal(root, _) if matches!(root.as_ref(), Kind::Array(_)) => {
                    let Kind::Array(array) = root.as_ref() else {
                        return Err(rhdl_error(PathError::IndexingNotAllowed { kind: **root }));
                    };
                    let element_size = array.base.bits();
                    if i >= &array.size {
                        return Err(rhdl_error(PathError::ArrayIndexOutOfBounds {
                            ndx: *i,
                            kind,
                        }));
                    }
                    range = range.start + i * element_size..range.start + (i + 1) * element_size;
                    kind = *array.base;
                }
                _ => return Err(rhdl_error(PathError::IndexingNotAllowed { kind })),
            },
            PathElement::Field(field) => match &kind {
                Kind::Struct(structure) => {
                    if !structure.fields.iter().any(|f| &f.name == field) {
                        return Err(rhdl_error(PathError::FieldNotFound {
                            field: *field,
                            kind,
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
                    kind = *field;
                }
                _ => return Err(rhdl_error(PathError::FieldIndexingNotAllowed { kind })),
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
                            rhdl_error(PathError::EnumPayloadNotFound { name: *name, kind })
                        })?
                        .kind;
                    range = match enumerate.discriminant_layout.alignment {
                        DiscriminantAlignment::Lsb => {
                            range.start + enumerate.discriminant_layout.width
                                ..range.start + enumerate.discriminant_layout.width + field.bits()
                        }
                        DiscriminantAlignment::Msb => range.start..range.start + field.bits(),
                    };
                    kind = field;
                }
                _ => return Err(rhdl_error(PathError::EnumPayloadNotValid { kind })),
            },
            PathElement::EnumPayloadByValue(disc) => match &kind {
                Kind::Enum(enumerate) => {
                    let field = enumerate
                        .variants
                        .iter()
                        .find(|f| f.discriminant == *disc)
                        .ok_or_else(|| {
                            rhdl_error(PathError::EnumPayloadByValueNotFound { disc: *disc, kind })
                        })?
                        .kind;
                    range = match enumerate.discriminant_layout.alignment {
                        DiscriminantAlignment::Lsb => {
                            range.start + enumerate.discriminant_layout.width
                                ..range.start + enumerate.discriminant_layout.width + field.bits()
                        }
                        DiscriminantAlignment::Msb => range.start..range.start + field.bits(),
                    };
                    kind = field;
                }
                _ => return Err(rhdl_error(PathError::EnumPayloadByValueNotValid { kind })),
            },
            PathElement::DynamicIndex(_slot) => {
                return Err(rhdl_error(PathError::DynamicIndicesNotResolved {
                    path: path.clone(),
                }));
            }
        }
    }
    Ok((range, kind))
}

#[cfg(test)]
mod tests {
    use crate::{Kind, types::kind::DiscriminantLayout};

    use super::{Path, leaf_paths};

    #[test]
    fn test_leaf_path() {
        let base_struct = Kind::make_struct(
            "base",
            vec![
                Kind::make_field("a", Kind::make_bits(8)),
                Kind::make_field("b", Kind::make_array(Kind::make_bits(8), 3)),
            ]
            .into(),
        );
        // Create a path with a struct, containing and array of structs
        let lev2 = Kind::make_struct(
            "foo",
            vec![
                Kind::make_field("c", base_struct),
                Kind::make_field("d", Kind::make_array(base_struct, 4)),
            ]
            .into(),
        );
        let kind = Kind::make_enum(
            "bar",
            vec![
                Kind::make_variant("a", Kind::make_bits(8), 0),
                Kind::make_variant("b", lev2, 1),
            ],
            DiscriminantLayout {
                width: 8,
                alignment: crate::DiscriminantAlignment::Lsb,
                ty: crate::DiscriminantType::Unsigned,
            },
        );
        let mut bit_mask = vec![false; kind.bits()];
        for path in leaf_paths(&kind, Path::default()) {
            let (range, _) = super::bit_range(kind, &path).unwrap();
            for i in range {
                bit_mask[i] = true;
            }
        }
        assert!(bit_mask.iter().all(|b| *b));
    }
}
