use crate::Kind;
use anyhow::{bail, Result};

// This is an enum that is a wrapper around the Kind type but
// that keeps track of reference semantics (i.e., taking addresses),
// extracting fields, etc.
#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Kind(Kind),
    Address(Kind),
    Empty,
}

impl Ty {
    pub fn kind(&self) -> Result<Kind> {
        match self {
            Ty::Kind(kind) => Ok(kind.clone()),
            Ty::Address(_) => bail!("Expected kind, found address"),
            Ty::Empty => bail!("Expected kind, found empty"),
        }
    }
    pub fn target_kind(&self) -> Result<Kind> {
        match self {
            Ty::Kind(_) => bail!("Expected address, found kind"),
            Ty::Address(kind) => Ok(kind.clone()),
            Ty::Empty => bail!("Expected address, found empty"),
        }
    }
}
