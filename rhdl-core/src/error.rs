use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum RHDLError {
    #[error("Internal Error: {0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("Unparseable integer error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("RHDL Syntax Error")]
    #[diagnostic(transparent)]
    RHDLSyntaxError(#[from] crate::compiler::mir::error::RHDLSyntaxError),
    #[error("RHDL ICE")]
    #[diagnostic(transparent)]
    RHDLInternalCompilerError(#[from] crate::compiler::mir::error::RHDLCompileError),
    #[error("RHDL Type Check Error")]
    #[diagnostic(transparent)]
    RHDLTypeError(#[from] crate::compiler::mir::error::RHDLTypeError),
}
