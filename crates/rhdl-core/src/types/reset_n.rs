//! An active Low reset signal type
//!
//! This is a newtype wrapper around a boolean to represent active low reset signals.
//! The sense of the signal is active low - meaning that when the signal
//! is false, the reset is active.  You can test for a reset in your kernels
//! using the `.any()` and `.all()` methods, and then negating the result.
use crate::{Digital, Kind, bitx::BitX};

/// An active low reset type
#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub struct ResetN(bool);

impl ResetN {
    /// Get the raw boolean value of the reset.
    #[must_use]
    pub fn raw(&self) -> bool {
        self.0
    }
    /// Returns true if the reset is high (not active).
    #[must_use]
    pub fn any(self) -> bool {
        self.0
    }
    /// Returns true if the reset is high (not active).
    pub fn all(self) -> bool {
        self.0
    }
}

/// Create an active low reset from a boolean.
///
/// This is not a synthesizable function.  It's for testing.
pub fn reset_n(b: bool) -> ResetN {
    ResetN(b)
}

impl Digital for ResetN {
    const BITS: usize = 1;
    fn static_kind() -> Kind {
        Kind::make_bool()
    }
    fn bin(self) -> Box<[BitX]> {
        [self.0.into()].into()
    }
    fn dont_care() -> Self {
        ResetN(true)
    }
}
