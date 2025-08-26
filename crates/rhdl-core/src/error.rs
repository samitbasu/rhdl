use miette::Diagnostic;
use thiserror::Error;

use crate::rhdl_core::{
    KernelFnKind, TypedBits,
    circuit::yosys::YosysSynthError,
    compiler::mir::ty::UnifyError,
    types::{bit_string::BitString, path::PathError},
};

#[derive(Error, Debug, Diagnostic)]
pub enum RHDLError {
    #[error("Internal Error: {0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("Unparseable integer error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("RHDL Syntax Error")]
    #[diagnostic(transparent)]
    RHDLSyntaxError(#[from] Box<crate::rhdl_core::compiler::mir::error::RHDLSyntaxError>),
    #[error("RHDL ICE")]
    #[diagnostic(transparent)]
    RHDLInternalCompilerError(
        #[from] Box<crate::rhdl_core::compiler::mir::error::RHDLCompileError>,
    ),
    #[error("RHDL Type Error")]
    #[diagnostic(transparent)]
    RHDLTypeError(#[from] Box<crate::rhdl_core::compiler::mir::error::RHDLTypeError>),
    #[error("RHDL Type Check Error")]
    #[diagnostic(transparent)]
    RHDLTypeCheckError(#[from] Box<crate::rhdl_core::compiler::mir::error::RHDLTypeCheckError>),
    #[error("RHDL Partial Initialization Error")]
    #[diagnostic(transparent)]
    RHDLPartialInitializationError(
        #[from] Box<crate::rhdl_core::compiler::mir::error::RHDLPartialInitializationError>,
    ),
    #[error("RHDL Clock Domain Violation")]
    #[diagnostic(transparent)]
    RHDLClockDomainViolation(
        #[from] Box<crate::rhdl_core::compiler::mir::error::RHDLClockDomainViolation>,
    ),
    #[error("RHDL Dynamic Type Error")]
    #[diagnostic(transparent)]
    RHDLDynamicTypeError(#[from] Box<crate::rhdl_core::types::error::DynamicTypeError>),
    #[error("RHDL Path Error")]
    #[diagnostic(transparent)]
    RHDLErrorPath(#[from] Box<PathError>),
    #[error("IO Error")]
    IOError(#[from] std::io::Error),
    #[error("Verilog Verification Error in RHIF: Expected {expected:?} got {actual:?}")]
    VerilogVerificationErrorTyped {
        expected: TypedBits,
        actual: TypedBits,
    },
    #[error("Cannot convert kernel function to Verilog descriptor {value:?}")]
    CannotConvertKernelFunctionToVerilogDescriptor { value: Box<KernelFnKind> },
    #[error("Verilog Verification Error in RTL: Expected {expected:?} got {actual:?}")]
    VerilogVerificationErrorRTL {
        expected: BitString,
        actual: BitString,
    },
    #[error("Verilog verification error: {0}")]
    VerilogVerificationErrorString(String),
    #[error("Testbench Construction Error: {0}")]
    TestbenchConstructionError(String),
    #[error("Circuits with no outputs are not synthesizable")]
    NoOutputsError,
    #[error("syn parsing error: {0}")]
    SynError(#[from] syn::Error),
    #[error("Top module export error: {0}")]
    ExportError(#[from] crate::rhdl_core::circuit::fixture::ExportError),
    #[error("This module is not synthesizable")]
    NotSynthesizable,
    #[error("Yosys synthesis error: {0}")]
    YosysSynthError(#[from] YosysSynthError),
    #[error("Netlist Error")]
    #[diagnostic(transparent)]
    NetListError(#[from] Box<crate::rhdl_core::ntl::error::NetListError>),
    #[error("Logic Loop")]
    #[diagnostic(transparent)]
    NetLoopError(#[from] Box<crate::rhdl_core::ntl::error::NetLoopError>),
    #[error("Type Inference Error")]
    #[diagnostic(transparent)]
    TypeInferenceError(#[from] Box<UnifyError>),
}

pub fn rhdl_error<T>(error: T) -> RHDLError
where
    RHDLError: From<Box<T>>,
{
    Box::new(error).into()
}
