pub use rhdl_typenum::prelude::*;

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
