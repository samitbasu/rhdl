use seq_macro::seq;

use crate::consts::U0;
use crate::{digits::*, Trimmed};
use crate::{
    traits::{Digit, Len, Unsigned},
    Trim,
};

impl Unsigned for UTerm {}

// Define the terminal symbol
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug, Default)]
pub struct UTerm;

impl UTerm {
    pub fn new() -> Self {
        Self
    }
}

// Define an unsigned integer
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug, Default)]
pub struct UInt<U, B> {
    pub msb: U,
    pub lsb: B,
}

impl<U: Unsigned, B: Digit> UInt<U, B> {
    #[inline]
    pub fn new() -> UInt<U, B> {
        UInt::default()
    }
}

impl Len for UTerm {
    type Output = U0;
    fn len(&self) -> Self::Output {
        UTerm
    }
}

impl<U: Unsigned, B: Digit> Unsigned for UInt<U, B> {
    const USIZE: usize = U::USIZE * 10 + B::USIZE;
}

impl Trim for UInt<UTerm, D0> {
    type Output = UTerm;
    fn trim(&self) -> Self::Output {
        UTerm
    }
}

seq!(N in 1..=9 {
    impl Trim for UInt<UTerm, D~N> {
        type Output = UInt<UTerm, D~N>;
        fn trim(&self) -> Self::Output {
            UInt::new()
        }
    }
});

impl<U, B: Digit> Trim for UInt<U, B>
where
    U: Trim,
{
    type Output = UInt<Trimmed<U>, B>;
    fn trim(&self) -> Self::Output {
        UInt::new()
    }
}
