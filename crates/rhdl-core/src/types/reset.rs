//! A reset type
//!
//! This is a newtype wrapper around a boolean to represent reset signals.
//! The sense of the signal is active high - meaning that when the signal
//! is true, the reset is active.  You can test for a reset in your kernels
//! using the `.any()` and `.all()` methods.
use crate::{Digital, Kind, bitx::BitX};

/// A reset type.
#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub struct Reset(bool);

impl Reset {
    /// Get the raw boolean value of the reset.
    #[must_use]
    pub fn raw(&self) -> bool {
        self.0
    }
    /// Returns true if the reset is active.
    #[must_use]
    pub fn any(self) -> bool {
        self.0
    }
    /// Returns true if the reset is active.
    #[must_use]
    pub fn all(self) -> bool {
        self.0
    }
}

/// Create a reset from a boolean.
///
/// This is not a synthesizable function.  It's for testing.
#[must_use]
pub fn reset(b: bool) -> Reset {
    Reset(b)
}

impl Digital for Reset {
    const BITS: usize = 1;
    fn static_kind() -> Kind {
        Kind::make_bool()
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rhdl_trace_type::TraceType::Reset
    }
    fn bin(self) -> Box<[BitX]> {
        [self.0.into()].into()
    }
    fn dont_care() -> Self {
        Reset(false)
    }
}
