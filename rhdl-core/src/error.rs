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
    #[error("RHDL Clock Coherence Violation")]
    #[diagnostic(transparent)]
    RHDLClockCoherenceViolation(
        #[from] Box<crate::compiler::mir::error::RHDLClockCoherenceViolation>,
    ),
}
