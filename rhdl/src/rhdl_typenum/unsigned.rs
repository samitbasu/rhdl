use super::digits::Digit;

pub trait Unsigned: Copy + Default + 'static {
    const USIZE: usize = 0;
    fn new() -> Self {
        Self::default()
    }
}

impl Unsigned for T_ {}

// Define the terminal symbol
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug, Default)]
pub struct T_;

impl T_ {
    pub fn new() -> Self {
        Self
    }
}

// Define an unsigned integer
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug, Default)]
pub struct U_<U, B> {
    pub msb: U,
    pub lsb: B,
}

impl<U: Unsigned, B: Digit> U_<U, B> {
    #[inline]
    pub fn new() -> U_<U, B> {
        U_::default()
    }
}

impl<U: Unsigned, B: Digit> Unsigned for U_<U, B> {
    const USIZE: usize = U::USIZE * 10 + B::DIGIT_USIZE;
}
