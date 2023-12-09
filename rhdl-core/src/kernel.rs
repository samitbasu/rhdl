use anyhow::bail;
use serde::{Deserialize, Serialize};

use crate::ast::{self, KernelFn};

#[derive(Debug, Clone)]
pub struct Kernel {
    pub ast: Box<ast::KernelFn>,
}

impl From<Box<ast::KernelFn>> for Kernel {
    fn from(ast: Box<ast::KernelFn>) -> Self {
        Kernel { ast }
    }
}

impl TryFrom<KernelFnKind> for Kernel {
    type Error = anyhow::Error;

    fn try_from(kind: KernelFnKind) -> Result<Self, Self::Error> {
        match kind {
            KernelFnKind::Kernel(kernel) => Ok(Kernel { ast: kernel }),
            _ => bail!("Cannot convert non-AST kernel to AST kernel"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KernelFnKind {
    Kernel(Box<KernelFn>),
    Extern(ExternalKernelDef),
    TupleStructConstructor,
    BitConstructor(usize),
    SignedBitsConstructor(usize),
    MethodCall(MethodCallDef),
}

impl std::fmt::Display for KernelFnKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KernelFnKind::Kernel(kernel) => write!(f, "kernel"),
            KernelFnKind::Extern(extern_kernel) => write!(f, "extern {}", extern_kernel.name),
            KernelFnKind::TupleStructConstructor => write!(f, "tuple struct constructor"),
            KernelFnKind::BitConstructor(width) => write!(f, "bit constructor {}", width),
            KernelFnKind::SignedBitsConstructor(width) => {
                write!(f, "signed bits constructor {}", width)
            }
            KernelFnKind::MethodCall(method_call) => write!(f, "method call"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalKernelDef {
    pub name: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MethodCallDef {
    Any(usize),
}
