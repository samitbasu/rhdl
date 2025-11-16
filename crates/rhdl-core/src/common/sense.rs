//! Common definition of a Read/Write sense
//!
//! This module defines the `Sense` enum, which represents whether a particular
//! operation is a read or a write.  This is useful for various parts of the RHDL
//! framework that need to distinguish between reading from and writing to registers.

/// Used to mark an operation as a read or a write.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sense {
    /// Read operation
    Read,
    /// Write operation
    Write,
}

impl Sense {
    /// Returns true if the sense is read.
    #[must_use]
    pub fn is_read(&self) -> bool {
        matches!(self, Sense::Read)
    }
    /// Returns true if the sense is write.
    #[must_use]
    pub fn is_write(&self) -> bool {
        matches!(self, Sense::Write)
    }
}
