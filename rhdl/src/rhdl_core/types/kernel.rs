use crate::rhdl_core::{
    ast::ast_impl::{self, WrapOp},
    Color, TypedBits,
};

#[derive(Debug, Clone, Hash)]
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

#[derive(Clone, Hash)]
pub enum KernelFnKind {
    Kernel(Kernel),
    TupleStructConstructor(TypedBits),
    BitConstructor(usize),
    SignedBitsConstructor(usize),
    EnumTupleStructConstructor(TypedBits),
    SignalConstructor(Option<Color>),
    BitCast(usize),
    SignedCast(usize),
    Wrap(WrapOp),
}

impl std::fmt::Debug for KernelFnKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KernelFnKind::Kernel(kernel) => {
                write!(
                    f,
                    "kernel {name} {fn_id:?}",
                    name = kernel.inner().name,
                    fn_id = kernel.inner().fn_id
                )
            }
            KernelFnKind::TupleStructConstructor(tb) => {
                write!(f, "tuple struct constructor {:?}", tb)
            }
            KernelFnKind::BitConstructor(width) => write!(f, "bit constructor {}", width),
            KernelFnKind::SignedBitsConstructor(width) => {
                write!(f, "signed bits constructor {}", width)
            }
            KernelFnKind::EnumTupleStructConstructor(tb) => {
                write!(f, "enum tuple struct constructor {:?}", tb)
            }
            KernelFnKind::SignalConstructor(color) => {
                write!(f, "signal constructor {:?}", color)
            }
            KernelFnKind::BitCast(width) => write!(f, "bit cast {}", width),
            KernelFnKind::SignedCast(width) => write!(f, "signed cast {}", width),
            KernelFnKind::Wrap(op) => write!(f, "wrap {:?}", op),
        }
    }
}
