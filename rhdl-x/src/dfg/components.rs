use rhdl_core::{
    path::Path,
    rhif::spec::{AluBinary, AluUnary, CaseArgument, Member},
    TypedBits,
};

use super::schematic::PinIx;

#[derive(Clone, Debug)]
pub struct Component {
    pub name: String,
    pub kind: ComponentKind,
}

#[derive(Clone, Debug)]
pub struct BinaryComponent {
    pub op: AluBinary,
    pub input1: PinIx,
    pub input2: PinIx,
    pub output: PinIx,
}

#[derive(Clone, Debug)]
pub struct UnaryComponent {
    pub op: AluUnary,
    pub input: PinIx,
    pub output: PinIx,
}

#[derive(Clone, Debug)]
pub struct SelectComponent {
    pub cond: PinIx,
    pub true_value: PinIx,
    pub false_value: PinIx,
    pub output: PinIx,
}

#[derive(Clone, Debug)]
pub struct IndexComponent {
    pub arg: PinIx,
    pub path: Path,
    pub output: PinIx,
    pub dynamic: Vec<PinIx>,
}

#[derive(Clone, Debug)]
pub struct SpliceComponent {
    pub orig: PinIx,
    pub subst: PinIx,
    pub output: PinIx,
    pub path: Path,
    pub dynamic: Vec<PinIx>,
}

#[derive(Clone, Debug)]
pub struct RepeatComponent {
    pub value: PinIx,
    pub output: PinIx,
    pub len: usize,
}

#[derive(Clone, Debug)]
pub struct FieldPin {
    pub member: Member,
    pub pin: PinIx,
}

#[derive(Clone, Debug)]
pub struct StructComponent {
    pub fields: Vec<FieldPin>,
    pub output: PinIx,
    pub rest: Option<PinIx>,
}

#[derive(Clone, Debug)]
pub struct TupleComponent {
    pub fields: Vec<PinIx>,
    pub output: PinIx,
}

#[derive(Clone, Debug)]
pub struct CaseComponent {
    pub discriminant: PinIx,
    pub table: Vec<(CaseArgument, PinIx)>,
    pub output: PinIx,
}

#[derive(Clone, Debug)]
pub struct ExecComponent {
    pub name: String,
    pub args: Vec<PinIx>,
    pub output: PinIx,
}

#[derive(Clone, Debug)]
pub struct ArrayComponent {
    pub elements: Vec<PinIx>,
    pub output: PinIx,
}

#[derive(Clone, Debug)]
pub struct DiscriminantComponent {
    pub arg: PinIx,
    pub output: PinIx,
}

#[derive(Clone, Debug)]
pub struct EnumComponent {
    pub fields: Vec<FieldPin>,
    pub output: PinIx,
}

#[derive(Clone, Debug)]
pub struct BufferComponent {
    pub input: PinIx,
    pub output: PinIx,
}

#[derive(Clone, Debug)]
pub struct CastComponent {
    pub input: PinIx,
    pub output: PinIx,
}

#[derive(Clone, Debug)]
pub struct ConstantComponent {
    pub value: TypedBits,
    pub output: PinIx,
}

#[derive(Clone, Debug)]
pub enum ComponentKind {
    Buffer(BufferComponent),
    Binary(BinaryComponent),
    Unary(UnaryComponent),
    Select(SelectComponent),
    Index(IndexComponent),
    Splice(SpliceComponent),
    Repeat(RepeatComponent),
    Struct(StructComponent),
    Tuple(TupleComponent),
    Case(CaseComponent),
    Exec(ExecComponent),
    Array(ArrayComponent),
    Discriminant(DiscriminantComponent),
    Enum(EnumComponent),
    Constant(ConstantComponent),
    Cast(CastComponent),
}
