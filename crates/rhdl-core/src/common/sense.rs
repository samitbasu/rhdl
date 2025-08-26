#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sense {
    Read,
    Write,
}

impl Sense {
    pub fn is_read(&self) -> bool {
        matches!(self, Sense::Read)
    }

    pub fn is_write(&self) -> bool {
        matches!(self, Sense::Write)
    }
}
