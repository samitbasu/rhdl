use crate::rhdl_bits::{BitWidth, Bits, SignedBits};

use crate::rhdl_core::{Digital, Kind};

pub trait Register: Digital {
    fn static_kind() -> Kind {
        <Self as Digital>::static_kind()
    }
}

impl<N: BitWidth> Register for Bits<N> {}

impl<N: BitWidth> Register for SignedBits<N> {}

pub trait SignedRegister: Digital {
    fn static_kind() -> Kind {
        <Self as Digital>::static_kind()
    }
}

impl<N: BitWidth> SignedRegister for SignedBits<N> {}
