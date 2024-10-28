use rhdl_bits::Bits;

use crate::{Notable, NoteKey, NoteWriter};

#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub struct BitZ<const N: usize> {
    pub value: Bits<N>,
    pub mask: Bits<N>,
}

impl<const N: usize> Notable for BitZ<N> {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        writer.write_tristate(key, self.value.0, self.mask.0, N as u8);
    }
}
