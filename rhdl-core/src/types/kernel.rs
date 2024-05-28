use crate::{ast::ast_impl, ClockColor, Kind, TypedBits};

#[derive(Debug, Clone)]
pub struct Kernel(Box<ast_impl::KernelFn>);

impl From<Box<ast_impl::KernelFn>> for Kernel {
    fn from(ast: Box<ast_impl::KernelFn>) -> Self {
        Kernel(ast)
    }
}

impl Kernel {
    pub fn inner(&self) -> &ast_impl::KernelFn {
        &self.0
    }
    pub fn inner_mut(&mut self) -> &mut ast_impl::KernelFn {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub enum KernelFnKind {
    Kernel(Kernel),
    Extern(ExternalKernelDef),
    TupleStructConstructor(TypedBits),
    BitConstructor(usize),
    SignedBitsConstructor(usize),
    EnumTupleStructConstructor(TypedBits),
    SignalConstructor(Option<ClockColor>),
}

impl std::fmt::Display for KernelFnKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KernelFnKind::Kernel(kernel) => {
                write!(
                    f,
                    "kernel {name} {fn_id}",
                    name = kernel.inner().name,
                    fn_id = kernel.inner().fn_id
                )
            }
            KernelFnKind::Extern(extern_kernel) => write!(f, "extern {}", extern_kernel.name),
            KernelFnKind::TupleStructConstructor(tb) => {
                write!(f, "tuple struct constructor {}", tb)
            }
            KernelFnKind::BitConstructor(width) => write!(f, "bit constructor {}", width),
            KernelFnKind::SignedBitsConstructor(width) => {
                write!(f, "signed bits constructor {}", width)
            }
            KernelFnKind::EnumTupleStructConstructor(tb) => {
                write!(f, "enum tuple struct constructor {}", tb)
            }
            KernelFnKind::SignalConstructor(color) => {
                write!(f, "signal constructor {:?}", color)
            }
        }
    }
}

type VMFunction = fn(&[TypedBits]) -> anyhow::Result<TypedBits>;

#[derive(Debug, Clone)]
pub struct ExternalKernelDef {
    pub name: String,
    pub body: String,
    pub vm_stub: Option<VMFunction>,
}
