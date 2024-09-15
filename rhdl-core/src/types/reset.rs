use crate::{Digital, Kind, Notable, NoteKey, NoteWriter};

#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub struct Reset(bool);

impl Reset {
    pub fn raw(&self) -> bool {
        self.0
    }
    pub fn any(self) -> bool {
        self.0
    }
    pub fn all(self) -> bool {
        self.0
    }
}

pub fn reset(b: bool) -> Reset {
    Reset(b)
}

impl Digital for Reset {
    fn static_kind() -> Kind {
        Kind::make_bool()
    }
    fn bin(self) -> Vec<bool> {
        vec![self.0]
    }
    fn init() -> Self {
        Reset(false)
    }
}

impl Notable for Reset {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        writer.write_bool(key, self.0);
    }
}
