//! A clock type.
//!
//! This type is a newtype wrapper around a boolean to represent clock signals.
use crate::{Digital, Kind, bitx::BitX};

/// A clock type.
#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub struct Clock(bool);

impl Clock {
    /// Get the raw boolean value of the clock.
    #[must_use]
    pub fn raw(&self) -> bool {
        self.0
    }
}

/// Create a clock from a boolean.
///
/// This is not a synthesizable function.  It's for testing.
pub fn clock(b: bool) -> Clock {
    Clock(b)
}

impl Digital for Clock {
    const BITS: usize = 1;
    fn static_kind() -> Kind {
        Kind::Clock
    }
    fn bin(self) -> Box<[BitX]> {
        [self.0.into()].into()
    }
    fn dont_care() -> Self {
        Clock(false)
    }
}
