pub use rhdl_typenum::prelude::*;

pub trait BitWidth: Copy + Clone + Default + PartialEq + Eq + 'static {
    const BITS: usize;
}

impl<N> BitWidth for N
where
    N: Unsigned + Copy + Clone + Default + PartialEq + Eq + 'static,
{
    const BITS: usize = N::USIZE;
}

#[cfg(test)]
mod tests {
    use super::BitWidth;
    use rhdl_typenum::prelude::*;
    use seq_macro::seq;

    // Check that Const<N> : BitWidth for all N s.t. N <= 128
    #[test]
    fn test_const_bitwidth() {
        seq!(N in 1..=128 {
            assert_eq!(<Const<N> as BitWidth>::BITS, N);
        });
    }
}
