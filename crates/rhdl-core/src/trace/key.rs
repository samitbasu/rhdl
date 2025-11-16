//! A trait for keys used to identify traced signals.
//!
//! This module defines a [TraceKey] trait that can be implemented on anything
//! that can be used as a key to identify traced signals.  The trait requires
//! the ability to clone, copy, and hash the key, as well as a method
//! to convert the key to a string representation.
//!
//! Implementations are provided for `&'static str`, `usize`,
//! slices of `&'static str`, and tuples of two TraceKeys.
//!
//! With these, you can call the [trace](crate::trace::db::trace) function with keys such as
//! `"signal_name"`, `42`, `&["module", "submodule", "signal"]`,
//! or `("module", "signal")`.
use std::hash::Hash;

/// A trait for keys used to identify traced signals.
pub trait TraceKey: Clone + Copy + Hash {
    /// Convert the key to a string representation.
    fn as_string(&self) -> String;
}

impl TraceKey for &'static str {
    fn as_string(&self) -> String {
        self.to_string()
    }
}

impl TraceKey for usize {
    fn as_string(&self) -> String {
        format!("{self}")
    }
}

impl TraceKey for &[&'static str] {
    fn as_string(&self) -> String {
        self.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join("::")
    }
}

impl<T: TraceKey, U: TraceKey> TraceKey for (T, U) {
    fn as_string(&self) -> String {
        format!("{}.{}", self.0.as_string(), self.1.as_string())
    }
}
