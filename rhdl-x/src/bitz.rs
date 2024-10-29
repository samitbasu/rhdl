use rhdl::prelude::*;

// TODO - Do not auto derive Notable....
#[derive(Debug, Clone, PartialEq, Copy, Default, Digital)]
pub struct Bitz<const N: usize> {
    pub value: Bits<N>,
    pub mask: Bits<N>,
}

impl<const N: usize> Notable for Bitz<N> {
    fn note(&self, key: impl NoteKey, mut writer: impl rhdl::core::NoteWriter) {
        writer.write_tristate(key, self.value.0, self.mask.0, N as u8);
    }
}
