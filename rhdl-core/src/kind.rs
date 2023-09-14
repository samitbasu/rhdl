#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Array(Array),
    Tuple(Tuple),
    Struct(Struct),
    Enum(Enum),
    Bits(usize),
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    pub base: Box<Kind>,
    pub size: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tuple {
    pub elements: Vec<Kind>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiscriminantAlignment {
    MSB,
    LSB,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub variants: Vec<Variant>,
    pub discriminant_width: Option<usize>,
    pub discriminant_alignment: DiscriminantAlignment,
}

impl Enum {
    fn discriminant_width(&self) -> usize {
        self.discriminant_width.unwrap_or_else(|| {
            self.variants
                .iter()
                .map(|x| x.discriminant)
                .max()
                .map(|x| clog2(x.max(1)))
                .unwrap_or(0)
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub kind: Kind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: String,
    pub discriminant: usize,
    pub kind: Kind,
}

pub const fn clog2(t: usize) -> usize {
    let mut p = 0;
    let mut b = 1;
    while b < t {
        p += 1;
        b *= 2;
    }
    p
}

impl Kind {
    pub fn make_array(base: Kind, size: usize) -> Self {
        Self::Array(Array {
            base: Box::new(base),
            size,
        })
    }
    pub fn make_tuple(elements: Vec<Kind>) -> Self {
        Self::Tuple(Tuple { elements })
    }
    pub fn make_struct(fields: Vec<Field>) -> Self {
        Self::Struct(Struct { fields })
    }
    pub fn make_enum(
        variants: Vec<Variant>,
        discriminant_width: Option<usize>,
        discriminant_alignment: DiscriminantAlignment,
    ) -> Self {
        Self::Enum(Enum {
            variants,
            discriminant_width,
            discriminant_alignment,
        })
    }
    pub fn make_bits(digits: usize) -> Self {
        Self::Bits(digits)
    }
    pub fn bits(&self) -> usize {
        match self {
            Kind::Array(array) => array.base.bits() * array.size,
            Kind::Tuple(tuple) => tuple.elements.iter().map(|x| x.bits()).sum(),
            Kind::Struct(kind) => kind.fields.iter().map(|x| x.kind.bits()).sum(),
            Kind::Enum(kind) => {
                kind.discriminant_width()
                    + kind
                        .variants
                        .iter()
                        .map(|x| x.kind.bits())
                        .max()
                        .unwrap_or(0)
            }
            Kind::Bits(digits) => *digits,
            Kind::Empty => 0,
        }
    }
}
