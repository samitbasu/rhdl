#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Array { base: Box<Kind>, size: usize },
    Tuple { elements: Vec<Kind> },
    Struct { fields: Vec<(String, Kind)> },
    Enum { variants: Vec<(String, Kind)> },
    Bits { digits: usize },
    Empty,
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
    pub fn bits(&self) -> usize {
        match self {
            Kind::Array { base, size } => base.bits() * size,
            Kind::Tuple { elements } => elements.iter().map(|x| x.bits()).sum(),
            Kind::Struct { fields } => fields.iter().map(|x| x.1.bits()).sum(),
            Kind::Enum { variants } => {
                clog2(variants.len()) + variants.iter().map(|x| x.1.bits()).max().unwrap_or(0)
            }
            Kind::Bits { digits } => *digits,
            Kind::Empty => 0,
        }
    }
}
