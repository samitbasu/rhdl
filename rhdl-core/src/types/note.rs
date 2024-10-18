// Essentially like a log for hardware designs that is designed more
// around time series of data values (like an oscilloscope) than a
// text journal of log entries.  The use of the name `note` is to
// suggest that it is _like_ a log, but not quite the same thing.
// The user can _also_ use regular log stuff if they want.

use std::hash::Hash;

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

pub trait NoteKey: Clone + Copy + Hash {
    fn as_string(&self) -> String;
}

impl NoteKey for &'static str {
    fn as_string(&self) -> String {
        self.to_string()
    }
}

impl NoteKey for usize {
    fn as_string(&self) -> String {
        format!("{}", self)
    }
}

impl NoteKey for &[&'static str] {
    fn as_string(&self) -> String {
        self.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join("::")
    }
}

impl<T: NoteKey, U: NoteKey> NoteKey for (T, U) {
    fn as_string(&self) -> String {
        format!("{}{}", self.0.as_string(), self.1.as_string())
    }
}

pub trait Notable {
    fn note(&self, key: impl NoteKey, writer: impl NoteWriter);
}

impl<T: Notable> Notable for &T {
    fn note(&self, key: impl NoteKey, writer: impl NoteWriter) {
        (*self).note(key, writer)
    }
}
