//! Traits used for logging values for debugging.
//!
//! See the [`crate::note_db`] module for more information on the logging system.
use std::hash::Hash;

/// Provides methods for logging key value pairs at the current simulation time.
pub trait NoteWriter {
    fn write_bool(&mut self, key: impl NoteKey, value: bool);
    fn write_bits(&mut self, key: impl NoteKey, value: u128, size: u8);
    fn write_signed(&mut self, key: impl NoteKey, value: i128, size: u8);
    fn write_string(&mut self, key: impl NoteKey, value: &'static str);
    fn write_tristate(&mut self, key: impl NoteKey, value: u128, mask: u128, size: u8);
}

impl<T: NoteWriter> NoteWriter for &mut T {
    fn write_bool(&mut self, key: impl NoteKey, value: bool) {
        (**self).write_bool(key, value)
    }
    fn write_bits(&mut self, key: impl NoteKey, value: u128, size: u8) {
        (**self).write_bits(key, value, size)
    }
    fn write_signed(&mut self, key: impl NoteKey, value: i128, size: u8) {
        (**self).write_signed(key, value, size)
    }
    fn write_string(&mut self, key: impl NoteKey, value: &'static str) {
        (**self).write_string(key, value)
    }
    fn write_tristate(&mut self, key: impl NoteKey, value: u128, mask: u128, size: u8) {
        (**self).write_tristate(key, value, mask, size)
    }
}

/// A value that can be used as a signal name in the vcd dump.
///
/// Used by the [`note`](crate::note) function.
pub trait NoteKey: Clone + Copy + Hash {
    fn as_string(&self) -> String;
}

/// A static string be used as is
impl NoteKey for &'static str {
    fn as_string(&self) -> String {
        self.to_string()
    }
}

/// Numbers are converted to strings
impl NoteKey for usize {
    fn as_string(&self) -> String {
        format!("{}", self)
    }
}

/// String arrays are be joined with `::`
impl NoteKey for &[&'static str] {
    fn as_string(&self) -> String {
        self.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join("::")
    }
}

/// Tuples of NoteKeys are joined with `::`
impl<T: NoteKey, U: NoteKey> NoteKey for (T, U) {
    fn as_string(&self) -> String {
        format!("{}::{}", self.0.as_string(), self.1.as_string())
    }
}

/// A value that can be logged to a [`NoteWriter`].
///
/// Used by the [`note`](crate::note) function.
pub trait Notable {
    /// Write this value to the note writer. The key is used to identify the value.
    fn note(&self, key: impl NoteKey, writer: impl NoteWriter);
}

impl<T: Notable> Notable for &T {
    fn note(&self, key: impl NoteKey, writer: impl NoteWriter) {
        (*self).note(key, writer)
    }
}
