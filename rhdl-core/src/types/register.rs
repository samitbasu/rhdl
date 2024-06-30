use rhdl_bits::{Bits, SignedBits};

use crate::{Digital, Kind};

pub trait Register: Digital {
    fn static_kind() -> Kind {
        <Self as Digital>::static_kind()
    }
}

impl<const N: usize> Register for Bits<N> {}

impl<const N: usize> Register for SignedBits<N> {}

pub trait SignedRegister: Digital {
    fn static_kind() -> Kind {
        <Self as Digital>::static_kind()
    }
}

impl<const N: usize> SignedRegister for SignedBits<N> {}