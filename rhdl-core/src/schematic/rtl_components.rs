use std::{iter::once, ops::Range};

use crate::{
    rhif::spec::{AluBinary, AluUnary},
    rtl::object::BitString,
};

use super::{db::PinIx, rtl_schematic::Schematic};

pub trait Component {
    fn inputs(&self) -> Vec<PinIx>;
    fn output(&self) -> PinIx;
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub bs: BitString,
    pub lhs: PinIx,
}

impl Component for Constant {
    fn inputs(&self) -> Vec<PinIx> {
        vec![]
    }
    fn output(&self) -> PinIx {
        self.lhs
    }
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub op: AluBinary,
    pub lhs: PinIx,
    pub arg1: PinIx,
    pub arg2: PinIx,
}

impl Component for Binary {
    fn inputs(&self) -> Vec<PinIx> {
        vec![self.arg1, self.arg2]
    }
    fn output(&self) -> PinIx {
        self.lhs
    }
}

#[derive(Debug, Clone)]
pub struct BlackBoxComponent {
    inputs: Vec<PinIx>,
    output: PinIx,
}

impl Component for BlackBoxComponent {
    fn inputs(&self) -> Vec<PinIx> {
        self.inputs.clone()
    }
    fn output(&self) -> PinIx {
        self.output
    }
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub op: AluUnary,
    pub lhs: PinIx,
    pub arg1: PinIx,
}

impl Component for Unary {
    fn inputs(&self) -> Vec<PinIx> {
        vec![self.arg1]
    }
    fn output(&self) -> PinIx {
        self.lhs
    }
}

#[derive(Debug, Clone)]
pub struct Select {
    pub lhs: PinIx,
    pub cond: PinIx,
    pub true_value: PinIx,
    pub false_value: PinIx,
}

impl Component for Select {
    fn inputs(&self) -> Vec<PinIx> {
        vec![self.cond, self.true_value, self.false_value]
    }
    fn output(&self) -> PinIx {
        self.lhs
    }
}

#[derive(Debug, Clone)]
pub struct Concat {
    pub lhs: PinIx,
    pub args: Vec<PinIx>,
}

impl Component for Concat {
    fn inputs(&self) -> Vec<PinIx> {
        self.args.clone()
    }
    fn output(&self) -> PinIx {
        self.lhs
    }
}

#[derive(Debug, Clone)]
pub struct DynamicIndex {
    pub lhs: PinIx,
    pub arg: PinIx,
    pub offset: PinIx,
    pub len: usize,
}

impl Component for DynamicIndex {
    fn inputs(&self) -> Vec<PinIx> {
        vec![self.arg, self.offset]
    }
    fn output(&self) -> PinIx {
        self.lhs
    }
}

#[derive(Debug, Clone)]
pub struct DynamicSplice {
    pub lhs: PinIx,
    pub arg: PinIx,
    pub offset: PinIx,
    pub len: usize,
    pub value: PinIx,
}

impl Component for DynamicSplice {
    fn inputs(&self) -> Vec<PinIx> {
        vec![self.arg, self.offset, self.value]
    }
    fn output(&self) -> PinIx {
        self.lhs
    }
}

#[derive(Debug, Clone)]
pub struct Index {
    pub lhs: PinIx,
    pub arg: PinIx,
    pub bit_range: Range<usize>,
}

impl Component for Index {
    fn inputs(&self) -> Vec<PinIx> {
        vec![self.arg]
    }
    fn output(&self) -> PinIx {
        self.lhs
    }
}

#[derive(Debug, Clone)]
pub struct Splice {
    pub lhs: PinIx,
    pub orig: PinIx,
    pub bit_range: Range<usize>,
    pub value: PinIx,
}

impl Component for Splice {
    fn inputs(&self) -> Vec<PinIx> {
        vec![self.orig, self.value]
    }
    fn output(&self) -> PinIx {
        self.lhs
    }
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub lhs: PinIx,
    pub rhs: PinIx,
}

impl Component for Assign {
    fn inputs(&self) -> Vec<PinIx> {
        vec![self.rhs]
    }
    fn output(&self) -> PinIx {
        self.lhs
    }
}

#[derive(Clone, Debug)]
pub enum CaseArgument {
    Literal(BitString),
    Wild,
}

#[derive(Clone, Debug)]
pub struct Case {
    pub lhs: PinIx,
    pub discriminant: PinIx,
    pub table: Vec<(CaseArgument, PinIx)>,
}

impl Component for Case {
    fn inputs(&self) -> Vec<PinIx> {
        once(self.discriminant)
            .chain(self.table.iter().map(|(_, pin)| *pin))
            .collect()
    }
    fn output(&self) -> PinIx {
        self.lhs
    }
}

#[derive(Debug, Clone)]
pub struct Cast {
    pub lhs: PinIx,
    pub arg: PinIx,
    pub len: usize,
    pub signed: bool,
}

impl Component for Cast {
    fn inputs(&self) -> Vec<PinIx> {
        vec![self.arg]
    }
    fn output(&self) -> PinIx {
        self.lhs
    }
}

#[derive(Debug, Clone)]
pub enum ComponentKind {
    Assign(Assign),
    Binary(Binary),
    BlackBox(BlackBoxComponent),
    Case(Case),
    Cast(Cast),
    Concat(Concat),
    Constant(Constant),
    DynamicIndex(DynamicIndex),
    DynamicSplice(DynamicSplice),
    Index(Index),
    Select(Select),
    Splice(Splice),
    Unary(Unary),
    Schematic(Box<Schematic>),
}

impl Component for ComponentKind {
    fn inputs(&self) -> Vec<PinIx> {
        match self {
            ComponentKind::Assign(assign) => assign.inputs(),
            ComponentKind::Binary(binary) => binary.inputs(),
            ComponentKind::BlackBox(blackbox) => blackbox.inputs(),
            ComponentKind::Case(case) => case.inputs(),
            ComponentKind::Cast(cast) => cast.inputs(),
            ComponentKind::Concat(concat) => concat.inputs(),
            ComponentKind::Constant(constant) => constant.inputs(),
            ComponentKind::DynamicIndex(dynamic_index) => dynamic_index.inputs(),
            ComponentKind::DynamicSplice(dynamic_splice) => dynamic_splice.inputs(),
            ComponentKind::Index(index) => index.inputs(),
            ComponentKind::Select(select) => select.inputs(),
            ComponentKind::Splice(splice) => splice.inputs(),
            ComponentKind::Unary(unary) => unary.inputs(),
            ComponentKind::Schematic(schematic) => schematic.inputs.clone(),
        }
    }
    fn output(&self) -> PinIx {
        match self {
            ComponentKind::Assign(assign) => assign.output(),
            ComponentKind::Binary(binary) => binary.output(),
            ComponentKind::BlackBox(blackbox) => blackbox.output(),
            ComponentKind::Case(case) => case.output(),
            ComponentKind::Cast(cast) => cast.output(),
            ComponentKind::Constant(constant) => constant.output(),
            ComponentKind::Concat(concat) => concat.output(),
            ComponentKind::DynamicIndex(dynamic_index) => dynamic_index.output(),
            ComponentKind::DynamicSplice(dynamic_splice) => dynamic_splice.output(),
            ComponentKind::Index(index) => index.output(),
            ComponentKind::Select(select) => select.output(),
            ComponentKind::Splice(splice) => splice.output(),
            ComponentKind::Unary(unary) => unary.output(),
            ComponentKind::Schematic(schematic) => schematic.output,
        }
    }
}
