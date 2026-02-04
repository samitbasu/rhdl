//! Atomic path indices

use crate::Kind;
use internment::Intern;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub(crate) struct AtomPath(pub Vec<AtomElement>);

impl AtomPath {
    pub(crate) fn index(mut self, index: usize) -> Self {
        self.0.push(AtomElement::Index(index));
        self
    }
    pub(crate) fn tuple_index(mut self, index: usize) -> Self {
        self.0.push(AtomElement::TupleIndex(index));
        self
    }
    pub(crate) fn field<S: Into<Intern<String>>>(mut self, name: S) -> Self {
        self.0.push(AtomElement::Field(name.into()));
        self
    }
    pub(crate) fn signal_value(mut self) -> Self {
        self.0.push(AtomElement::SignalValue);
        self
    }
    pub(crate) fn discriminant(mut self) -> Self {
        self.0.push(AtomElement::Discriminant);
        self
    }
    pub(crate) fn payload(mut self) -> Self {
        self.0.push(AtomElement::Payload);
        self
    }
    pub(crate) fn extend<I: IntoIterator<Item = AtomElement>>(mut self, elems: I) -> Self {
        self.0.extend(elems);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum AtomElement {
    /// An index into an array, `x[3]`
    Index(usize),
    /// An index into a tuple, `x.2`
    TupleIndex(usize),
    /// A field access into a struct, `x.field_name`
    Field(Intern<String>),
    /// The value of a signal, `x.value`
    SignalValue,
    /// The discriminant of an enum, `x.discriminant`
    Discriminant,
    /// The payload of an enum,
    Payload,
}

// Given a path and a kind, generate all atom paths starting
// at the given path - these are paths that terminate in
// non-composite elements of a data structure.  Note that
// an enumerated value is considered a two atoms, since
// you can access the discriminant and the payload.
pub(crate) fn atom_paths(kind: &Kind, base: AtomPath) -> Vec<AtomPath> {
    match kind {
        Kind::Array(array) => (0..array.size)
            .flat_map(|i| atom_paths(&array.base, base.clone().index(i)))
            .collect(),
        Kind::Tuple(tuple) => tuple
            .elements
            .iter()
            .enumerate()
            .flat_map(|(i, k)| atom_paths(k, base.clone().tuple_index(i)))
            .collect(),
        Kind::Struct(structure) => structure
            .fields
            .iter()
            .flat_map(|field| atom_paths(&field.kind, base.clone().field(field.name)))
            .collect(),
        Kind::Signal(root, _) => atom_paths(root, base.clone().signal_value()),
        Kind::Enum(_) => vec![base.clone().discriminant(), base.clone().payload()],
        Kind::Bits(_) | Kind::Signed(_) | Kind::Empty | Kind::Clock | Kind::Reset => {
            vec![base.clone()]
        }
    }
}

pub(crate) fn iter_atoms(kind: Kind) -> impl Iterator<Item = AtomPath> {
    let paths = atom_paths(&kind, AtomPath::default());
    paths.into_iter()
}
