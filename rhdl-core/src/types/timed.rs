use crate::{Digital, Kind, Notable};

pub trait Timed: Copy + Sized + PartialEq + Clone + 'static + Notable {
    fn static_kind() -> Kind;
    fn bits() -> usize {
        Self::static_kind().bits()
    }
    fn kind(&self) -> Kind {
        Self::static_kind()
    }
}

impl<T: Digital> Timed for T {
    fn static_kind() -> Kind {
        <T as Digital>::static_kind()
    }
}
