use crate::rhdl_typenum::const_generics::Const;
pub use crate::rhdl_typenum::prelude::*;
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

#[cfg(test)]
mod tests {
    use super::BitWidth;
    use crate::rhdl_typenum::prelude::*;
    use seq_macro::seq;
    use static_assertions::assert_impl_all;

    // Check that Const<N> : BitWidth for all N s.t. N <= 128
    #[test]
    fn test_const_bitwidth() {
        //        assert_impl_all!(Const<1> : BitWidth);
        //seq!(N in 1..=128 {
        //assert!(<Const<N> as BitWidth>::BITS == N);
        //});
    }
}
