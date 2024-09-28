use crate::{
    ast::source_location::SourceLocation,
    rhif::spec::{AluBinary, AluUnary},
    types::bit_string::BitString,
};

#[derive(Debug, Clone, Hash)]
pub struct Binary {
    pub op: AluBinary,
}

#[derive(Debug, Clone, Hash)]
pub struct Unary {
    pub op: AluUnary,
}

#[derive(Debug, Clone, Hash)]
pub struct DynamicIndex {
    pub len: usize,
}

#[derive(Debug, Clone, Hash)]
pub struct DynamicSplice {
    pub len: usize,
}

#[derive(Debug, Clone, Hash)]
pub struct BlackBox {
    pub name: String,
}

#[derive(Debug, Clone, Hash)]
pub struct Case {
    pub entries: Vec<CaseEntry>,
}

#[derive(Debug, Clone, Hash)]
pub enum CaseEntry {
    Literal(BitString),
    WildCard,
}

#[derive(Debug, Clone, Hash)]
pub enum ComponentKind {
    Binary(Binary),
    BlackBox(BlackBox),
    Buffer(String),
    Case(Case),
    Constant(bool),
    DynamicIndex(DynamicIndex),
    DynamicSplice(DynamicSplice),
    Select,
    Source(String),
    Sink(Sink),
    TimingStart,
    TimingEnd,
    Unary(Unary),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Sink {
    Data(String),
    Clock,
    Reset,
}

impl std::fmt::Display for Sink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sink::Data(name) => write!(f, "{}", name),
            Sink::Clock => write!(f, "clk"),
            Sink::Reset => write!(f, "rst"),
        }
    }
}

#[derive(Clone, Hash)]
pub struct Component {
    pub kind: ComponentKind,
    pub location: Option<SourceLocation>,
}

impl std::fmt::Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ComponentKind::Binary(binary) => write!(f, "{:?}", binary.op),
            ComponentKind::BlackBox(blackbox) => write!(f, "{}", blackbox.name),
            ComponentKind::Buffer(name) => write!(f, "{name}"),
            ComponentKind::Case(_) => write!(f, "Case"),
            ComponentKind::Constant(constant) => {
                write!(f, "const({})", if *constant { 1 } else { 0 })
            }
            ComponentKind::DynamicIndex(dynamic_index) => write!(f, "[[{}]]", dynamic_index.len),
            ComponentKind::DynamicSplice(dynamic_splice) => write!(f, "//{}//", dynamic_splice.len),
            ComponentKind::Select => write!(f, "?"),
            ComponentKind::Source(name) => write!(f, "src<{}>", name),
            ComponentKind::Sink(details) => write!(f, "sink<{}>", details),
            ComponentKind::TimingStart => write!(f, "timing_start"),
            ComponentKind::TimingEnd => write!(f, "timing_end"),
            ComponentKind::Unary(unary) => write!(f, "{:?}", unary.op),
        }?;
        writeln!(f)
    }
}
