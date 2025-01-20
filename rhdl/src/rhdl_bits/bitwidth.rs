use std::ops::Add;

pub use crate::rhdl_typenum::prelude::*;
use crate::rhdl_typenum::{bools::True, const_generics::Const};
use seq_macro::seq;

pub trait BitWidth: Copy + Clone + Default + PartialEq + Eq + 'static {
    const BITS: usize;
}

impl<N> BitWidth for N
where
    N: Unsigned + Copy + Clone + Default + PartialEq + Eq + 'static,
    N: IsLessThanOrEqual<U128>,
    IsLessThanOrEqualTo<N, U128>: IsTrue,
{
    const BITS: usize = N::USIZE;
}

seq!(N in 1..=128 {
    impl IsLessThanOrEqual<U128> for Const<N> {
        type Output = True;
    }
});

#[cfg(test)]
mod tests {
    use super::BitWidth;
    use crate::rhdl_typenum::prelude::*;
    use seq_macro::seq;

    // Check that Const<N> : BitWidth for all N s.t. N <= 128
    #[test]
    fn test_const_bitwidth() {
        seq!(N in 1..=128 {
            assert_eq!(<Const<N> as BitWidth>::BITS, N);
        });
    }
}
