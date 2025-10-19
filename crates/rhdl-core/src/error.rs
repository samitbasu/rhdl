use miette::Diagnostic;
use thiserror::Error;

use crate::{
    KernelFnKind,
    TypedBits,
    //circuit::yosys::YosysSynthError,
    compiler::mir::ty::UnifyError,
    types::path::PathError,
};

#[derive(Error, Debug, Diagnostic)]
pub enum RHDLError {
    #[error("Internal Error: {0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("Parsing Error: {0}")]
    #[diagnostic(transparent)]
    ParseError(#[from] rhdl_vlog::ParseError),
    #[error("Unparseable integer error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("RHDL Syntax Error")]
    #[diagnostic(transparent)]
    RHDLSyntaxError(#[from] Box<crate::compiler::mir::error::RHDLSyntaxError>),
    #[error("RHDL ICE")]
    #[diagnostic(transparent)]
    RHDLInternalCompilerError(#[from] Box<crate::compiler::mir::error::RHDLCompileError>),
    #[error("RHDL Type Error")]
    #[diagnostic(transparent)]
    RHDLTypeError(#[from] Box<crate::compiler::mir::error::RHDLTypeError>),
    #[error("RHDL Type Check Error")]
    #[diagnostic(transparent)]
    RHDLTypeCheckError(#[from] Box<crate::compiler::mir::error::RHDLTypeCheckError>),
    #[error("RHDL Partial Initialization Error")]
    #[diagnostic(transparent)]
    RHDLPartialInitializationError(
        #[from] Box<crate::compiler::mir::error::RHDLPartialInitializationError>,
    ),
    #[error("RHDL Clock Domain Violation")]
    #[diagnostic(transparent)]
    RHDLClockDomainViolation(#[from] Box<crate::compiler::mir::error::RHDLClockDomainViolation>),
    #[error("RHDL Dynamic Type Error")]
    #[diagnostic(transparent)]
    RHDLDynamicTypeError(#[from] Box<crate::types::error::DynamicTypeError>),
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
    VerilogVerificationErrorRTL { expected: String, actual: String },
    #[error("Verilog verification error: {0}")]
    VerilogVerificationErrorString(String),
    #[error("Testbench Construction Error: {0}")]
    TestbenchConstructionError(String),
    #[error("Circuits with no outputs are not synthesizable")]
    NoOutputsError,
    #[error("syn parsing error: {0}")]
    SynError(#[from] syn::Error),
    #[error("Top module export error: {0}")]
    ExportError(#[from] crate::circuit::fixture::ExportError),
    #[error("This module is not synthesizable")]
    NotSynthesizable,
    #[error("Netlist Error")]
    #[diagnostic(transparent)]
    NetListError(#[from] Box<crate::ntl::error::NetListError>),
    #[error("Logic Loop")]
    #[diagnostic(transparent)]
    NetLoopError(#[from] Box<crate::ntl::error::NetLoopError>),
    #[error("Type Inference Error")]
    #[diagnostic(transparent)]
    TypeInferenceError(#[from] Box<UnifyError>),
    #[error("Function is not synthesizable")]
    FunctionNotSynthesizable,
}

pub fn rhdl_error<T>(error: T) -> RHDLError
where
    RHDLError: From<Box<T>>,
{
    Box::new(error).into()
}
