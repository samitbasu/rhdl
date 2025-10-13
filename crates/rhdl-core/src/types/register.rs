use rhdl_bits::{BitWidth, Bits, SignedBits, W};

use crate::{Digital, Kind};

pub trait Register: Digital {
    fn static_kind() -> Kind {
        <Self as Digital>::static_kind()
    }
}

impl<const N: usize> Register for Bits<N> where W<N>: BitWidth {}

impl<const N: usize> Register for SignedBits<N> where W<N>: BitWidth {}

pub trait SignedRegister: Digital {
    fn static_kind() -> Kind {
        <Self as Digital>::static_kind()
    }
}

impl<const N: usize> SignedRegister for SignedBits<N> where W<N>: BitWidth {}
