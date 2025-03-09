use rhdl::{core::RHDLError, prelude::Path};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BspError {
    #[error("Path {0:?} on input is not a clock input")]
    NotAClockInput(Path),
    #[error("RHDL core error {0}")]
    RHDLError(#[from] RHDLError),
    #[error("Templating Error {0}")]
    TemplateError(#[from] tinytemplate::error::Error),
    #[error("Mismatch in signal width: expected {expected} bits, but input is {actual} bits wide")]
    SignalWidthMismatch { expected: i32, actual: usize },
}
