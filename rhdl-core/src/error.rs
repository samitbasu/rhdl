use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum RHDLError {
    #[error("MIR ICE Error: {0}")]
    ICE(#[from] crate::compiler::mir::error::ICE),
    #[error("MIR Syntax Error: {0}")]
    Syntax(#[from] crate::compiler::mir::error::Syntax),
    #[error("Internal Error: {0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("Unparseable integer error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
}
