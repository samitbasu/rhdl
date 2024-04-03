use crate::{
    path::Path,
    rhif::{
        object::SourceLocation,
        spec::{AluBinary, AluUnary, CaseArgument, Member},
    },
    Kind, TypedBits,
};
use anyhow::Result;
use std::sync::Arc;

use super::{
    constraints::Constraint,
    schematic_impl::{PinIx, PinPath, Schematic},
};

#[derive(Clone, Debug)]
pub struct Component {
    // TODO - worry about the string allocation...
    pub path: Vec<String>,
    pub kind: ComponentKind,
    pub location: Option<SourceLocation>,
}

impl Component {
    pub fn offset(self, path: &str, offset: usize) -> Component {
        // prefix the path list with the new path
        let mut path = vec![path.to_string()];
        path.extend(self.path);
        Component {
            path,
            kind: self.kind.offset(offset),
            location: self.location,
        }
    }
    pub fn is_noop(&self) -> bool {
        matches!(self.kind, ComponentKind::Noop)
    }
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
    pub kind: Kind,
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
    pub kind: Kind,
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
    pub kind: Kind,
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
pub struct KernelComponent {
    pub name: String,
    pub args: Vec<PinIx>,
    pub output: PinIx,
    pub sub_schematic: Schematic,
}

#[derive(Clone)]
pub struct BlackBoxComponent(pub Arc<dyn BlackBoxTrait>);

impl BlackBoxComponent {
    pub fn new(bb: impl BlackBoxTrait + 'static) -> Self {
        BlackBoxComponent(Arc::new(bb))
    }
}

impl BlackBoxTrait for BlackBoxComponent {
    fn name(&self) -> &str {
        self.0.name()
    }
    fn args(&self) -> Vec<PinIx> {
        self.0.args()
    }
    fn output(&self) -> PinIx {
        self.0.output()
    }
    fn upstream(&self, output: PinPath) -> Result<Vec<PinPath>> {
        self.0.upstream(output)
    }
    fn downstream(&self, input: PinPath) -> Result<Vec<PinPath>> {
        self.0.downstream(input)
    }
    fn offset(&self, shift: usize) -> BlackBoxComponent {
        BlackBoxComponent(Arc::new(self.0.offset(shift)))
    }
    fn constraints(&self) -> Vec<Constraint> {
        self.0.constraints()
    }
}

impl std::fmt::Debug for BlackBoxComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug)]
pub struct ArrayComponent {
    pub elements: Vec<PinIx>,
    pub output: PinIx,
}

#[derive(Clone, Debug)]
pub struct EnumComponent {
    pub fields: Vec<FieldPin>,
    pub output: PinIx,
    pub template: TypedBits,
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
    BlackBox(BlackBoxComponent),
    Kernel(KernelComponent),
    Array(ArrayComponent),
    Enum(EnumComponent),
    Constant(ConstantComponent),
    Cast(CastComponent),
    Noop,
}

impl ComponentKind {
    // Add an offset to all of the pins in the component.
    fn offset(self, offset: usize) -> Self {
        match self {
            ComponentKind::Buffer(mut c) => {
                c.input = c.input.offset(offset);
                c.output = c.output.offset(offset);
                ComponentKind::Buffer(c)
            }
            ComponentKind::Binary(mut c) => {
                c.input1 = c.input1.offset(offset);
                c.input2 = c.input2.offset(offset);
                c.output = c.output.offset(offset);
                ComponentKind::Binary(c)
            }
            ComponentKind::Unary(mut c) => {
                c.input = c.input.offset(offset);
                c.output = c.output.offset(offset);
                ComponentKind::Unary(c)
            }
            ComponentKind::Select(mut c) => {
                c.cond = c.cond.offset(offset);
                c.true_value = c.true_value.offset(offset);
                c.false_value = c.false_value.offset(offset);
                c.output = c.output.offset(offset);
                ComponentKind::Select(c)
            }
            ComponentKind::Index(mut c) => {
                c.arg = c.arg.offset(offset);
                c.output = c.output.offset(offset);
                c.dynamic = c.dynamic.iter().map(|p| p.offset(offset)).collect();
                ComponentKind::Index(c)
            }
            ComponentKind::Splice(mut c) => {
                c.orig = c.orig.offset(offset);
                c.subst = c.subst.offset(offset);
                c.output = c.output.offset(offset);
                c.dynamic = c.dynamic.iter().map(|p| p.offset(offset)).collect();
                ComponentKind::Splice(c)
            }
            ComponentKind::Repeat(mut c) => {
                c.value = c.value.offset(offset);
                c.output = c.output.offset(offset);
                ComponentKind::Repeat(c)
            }
            ComponentKind::Struct(mut c) => {
                c.output = c.output.offset(offset);
                c.rest = c.rest.map(|p| p.offset(offset));
                c.fields
                    .iter_mut()
                    .for_each(|f| f.pin = f.pin.offset(offset));
                ComponentKind::Struct(c)
            }
            ComponentKind::Tuple(mut c) => {
                c.output = c.output.offset(offset);
                c.fields = c.fields.iter().map(|p| p.offset(offset)).collect();
                ComponentKind::Tuple(c)
            }
            ComponentKind::Case(mut c) => {
                c.discriminant = c.discriminant.offset(offset);
                c.table.iter_mut().for_each(|(_, p)| *p = p.offset(offset));
                c.output = c.output.offset(offset);
                ComponentKind::Case(c)
            }
            ComponentKind::BlackBox(mut c) => {
                ComponentKind::BlackBox(c.0.offset(offset))
                //                c.args = c.args.iter().map(|p| p.offset(offset)).collect();
                //c.output = c.output.offset(offset);
                //ComponentKind::BlackBox(c)
            }
            ComponentKind::Kernel(mut c) => {
                c.args = c.args.iter().map(|p| p.offset(offset)).collect();
                c.output = c.output.offset(offset);
                ComponentKind::Kernel(c)
            }
            ComponentKind::Array(mut c) => {
                c.elements = c.elements.iter().map(|p| p.offset(offset)).collect();
                c.output = c.output.offset(offset);
                ComponentKind::Array(c)
            }
            ComponentKind::Enum(mut c) => {
                c.output = c.output.offset(offset);
                c.fields
                    .iter_mut()
                    .for_each(|f| f.pin = f.pin.offset(offset));
                ComponentKind::Enum(c)
            }
            ComponentKind::Constant(mut c) => {
                c.output = c.output.offset(offset);
                ComponentKind::Constant(c)
            }
            ComponentKind::Cast(mut c) => {
                c.input = c.input.offset(offset);
                c.output = c.output.offset(offset);
                ComponentKind::Cast(c)
            }
            ComponentKind::Noop => ComponentKind::Noop,
        }
    }
}
pub trait BlackBoxTrait: core::fmt::Debug {
    fn name(&self) -> &str;
    fn args(&self) -> Vec<PinIx>;
    fn output(&self) -> PinIx;
    fn upstream(&self, output: PinPath) -> Result<Vec<PinPath>> {
        Ok(vec![])
    }
    fn downstream(&self, input: PinPath) -> Result<Vec<PinPath>> {
        Ok(vec![])
    }
    fn offset(&self, shift: usize) -> BlackBoxComponent;
    fn constraints(&self) -> Vec<Constraint> {
        vec![]
    }
}
