#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug, Default)]
pub struct True;

impl True {
    pub fn new() -> Self {
        Self
    }
}

pub trait Bool: Copy + Default + 'static {
    const BOOL: bool;
}

impl Bool for True {
    const BOOL: bool = true;
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug, Default)]
pub struct False;

impl False {
    pub fn new() -> Self {
        Self
    }
}

impl Bool for False {
    const BOOL: bool = false;
}

pub trait IsTrue {}

impl IsTrue for True {}

pub trait IsFalse {}

impl IsFalse for False {}
