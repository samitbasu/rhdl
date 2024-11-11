use crate::{
    ast::source_location::SourceLocation,
    hdl::ast::SignedWidth,
    rhif::spec::{AluBinary, AluUnary},
    types::bit_string::BitString,
};

#[derive(Debug, Clone, Hash)]
pub struct Binary {
    pub op: AluBinary,
    pub left_len: SignedWidth,
    pub right_len: SignedWidth,
}

#[derive(Debug, Clone, Hash)]
pub struct Unary {
    pub op: AluUnary,
    pub arg_len: SignedWidth,
}

#[derive(Debug, Clone, Hash)]
pub struct DynamicIndex {
    pub offset_len: usize,
    pub arg_len: usize,
}

#[derive(Debug, Clone, Hash)]
pub struct DynamicSplice {
    pub splice_len: usize,
    pub offset_len: usize,
}

#[derive(Debug, Clone, Hash)]
pub struct Case {
    pub entries: Vec<CaseEntry>,
    pub discriminant_width: SignedWidth,
}

#[derive(Debug, Clone, Hash)]
pub enum CaseEntry {
    Literal(BitString),
    WildCard,
}

#[derive(Debug, Clone, Hash)]
pub enum ComponentKind {
    Binary(Binary),
    BitSelect(BitSelect),
    BitString(BitString),
    Buffer(String),
    Case(Case),
    Constant(bool),
    BBInput(BBInput),
    BBOutput(BBOutput),
    DynamicIndex(DynamicIndex),
    DynamicSplice(DynamicSplice),
    Select,
    Unary(Unary),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BitSelect {
    pub bit_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BBInput {
    pub name: String,
    pub bit_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BBOutput {
    pub name: String,
    pub bit_index: usize,
}

#[derive(Clone, Hash)]
pub struct Component {
    pub kind: ComponentKind,
    pub width: usize,
    pub location: Option<SourceLocation>,
}

impl std::fmt::Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ComponentKind::Binary(binary) => write!(f, "{:?}", binary.op),
            ComponentKind::BitSelect(bit_select) => write!(f, "[{}]", bit_select.bit_index),
            ComponentKind::BitString(bit_string) => write!(f, "{:?}", bit_string),
            ComponentKind::Buffer(name) => write!(f, "{name}"),
            ComponentKind::Case(_) => write!(f, "Case"),
            ComponentKind::Constant(constant) => {
                write!(f, "{}", *constant as u8)
            }
            ComponentKind::DynamicIndex(dynamic_index) => {
                write!(f, "[[{} +: {}]]", dynamic_index.offset_len, self.width)
            }
            ComponentKind::DynamicSplice(dynamic_splice) => {
                write!(f, "//{}//", dynamic_splice.splice_len)
            }
            ComponentKind::Select => write!(f, "?"),
            ComponentKind::BBInput(dff_input) => {
                write!(f, "{}[{}]", dff_input.name, dff_input.bit_index)
            }
            ComponentKind::BBOutput(dff_output) => {
                write!(f, "{}[{}]", dff_output.name, dff_output.bit_index)
            }
            ComponentKind::Unary(unary) => write!(f, "{:?}", unary.op),
        }?;
        writeln!(f)
    }
}
