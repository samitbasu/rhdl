use std::ops::Range;

use crate::{
    rhif::spec::{AluBinary, AluUnary},
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
    Unary(Unary),
}
