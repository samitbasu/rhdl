use crate::{Digital, Kind};

#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub struct Clock(bool);

impl Clock {
    pub fn raw(&self) -> bool {
        self.0
    }
}

pub fn clock(b: bool) -> Clock {
    Clock(b)
}

impl Digital for Clock {
    const BITS: usize = 1;
    fn static_kind() -> Kind {
        Kind::make_bool()
    }
    fn bin(self) -> Vec<bool> {
        vec![self.0]
    }
    fn init() -> Self {
        Clock(false)
    }
}
