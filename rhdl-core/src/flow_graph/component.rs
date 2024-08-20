use std::ops::Range;

use crate::{
    rhif::{
        object::SourceLocation,
        spec::{AluBinary, AluUnary},
    },
    rtl::object::{BitString, RegisterKind},
};

#[derive(Debug, Clone)]
pub struct Buffer {
    pub kind: RegisterKind,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub bs: BitString,
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub op: AluBinary,
    pub width: usize,
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub op: AluUnary,
}

#[derive(Debug, Clone)]
pub struct DynamicIndex {
    pub len: usize,
}

#[derive(Debug, Clone)]
pub struct DynamicSplice {
    pub len: usize,
}

#[derive(Debug, Clone)]
pub struct Index {
    pub bit_range: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct Splice {
    pub bit_range: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct Cast {
    pub len: usize,
    pub signed: bool,
}

#[derive(Debug, Clone)]
pub struct BlackBox {
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum ComponentKind {
    Assign,
    Buffer(Buffer),
    Binary(Binary),
    BlackBox(BlackBox),
    Case,
    Cast(Cast),
    Concat,
    Constant(Constant),
    DynamicIndex(DynamicIndex),
    DynamicSplice(DynamicSplice),
    Index(Index),
    Select,
    Source(Buffer),
    Sink(Buffer),
    Splice(Splice),
    TimingStart,
    TimingEnd,
    Unary(Unary),
}

#[derive(Clone)]
pub struct Component {
    pub kind: ComponentKind,
    pub location: Option<SourceLocation>,
    pub cost: f64,
}

impl std::fmt::Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ComponentKind::Assign => write!(f, "v"),
            ComponentKind::Buffer(buffer) => write!(f, "{}", buffer.name),
            ComponentKind::Binary(binary) => write!(f, "{:?}<{}>", binary.op, binary.width),
            ComponentKind::BlackBox(blackbox) => write!(f, "{}", blackbox.name),
            ComponentKind::Case => write!(f, "Case"),
            ComponentKind::Cast(cast) => {
                if cast.signed {
                    write!(f, "as s{}", cast.len)
                } else {
                    write!(f, "as b{}", cast.len)
                }
            }
            ComponentKind::Concat => write!(f, "{{}}"),
            ComponentKind::Constant(constant) => write!(f, "{:?}", constant.bs),
            ComponentKind::DynamicIndex(dynamic_index) => write!(f, "[[{}]]", dynamic_index.len),
            ComponentKind::DynamicSplice(dynamic_splice) => write!(f, "//{}//", dynamic_splice.len),
            ComponentKind::Index(index) => {
                write!(f, "{}..{}", index.bit_range.start, index.bit_range.end)
            }
            ComponentKind::Select => write!(f, "?"),
            ComponentKind::Source(buffer) => write!(f, "src<{}>", buffer.name),
            ComponentKind::Sink(buffer) => write!(f, "sink<{}>", buffer.name),
            ComponentKind::Splice(splice) => {
                write!(f, "/{}..{}/", splice.bit_range.start, splice.bit_range.end)
            }
            ComponentKind::TimingStart => write!(f, "timing_start"),
            ComponentKind::TimingEnd => write!(f, "timing_end"),
            ComponentKind::Unary(unary) => write!(f, "{:?}", unary.op),
        }?;
        writeln!(f)?;
        writeln!(f, "{}", self.cost)
    }
}
