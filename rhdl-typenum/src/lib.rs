// Tell clippy to ignore this module
#![allow(clippy::all)]
use rhdl_macro::{add_impl, log2_impl, max_impl, min_impl, sub_impl};
use seq_macro::seq;

pub trait BitWidth {
    const BITS: usize;
}

// These are the type numbers that represent widths of the
// unsigned and signed bit types in RHDL.  I didn't use the
// underlying typenum::U types because they tend to make the
// error messages pretty difficult to read and also make
// rust-analyzer's type hints hard to read.  In this case, because
// we only need a finite number of widths, it seems feasible to
// just define them all as individual types, and then encode
// their relationships using procedural macros (essentially a
// table-driven approach).
seq!(N in 1..=128 {
    #(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        pub struct W~N;

        impl BitWidth for W~N {
            const BITS: usize = N;
        }
    )*
});

// Define the add trait for bit widths that do not exceed
// the maximum width of 128 bits.
seq!(N in 1..=128 {
    #(
    add_impl!(N);
    sub_impl!(N);
    max_impl!(N);
    min_impl!(N);
    log2_impl!(N);
    )*
});

#[cfg(test)]
mod tests {
    use typenum::{Diff, Log2, Maximum, Minimum, Sum, Unsigned};

    use typenum::consts::*;

    use super::*;

    #[test]
    fn test_log2() {
        let g = Log2::<U1>::to_usize();
        assert_eq!(g, 0);
        let g = Log2::<U8>::to_usize();
        assert_eq!(g, 3);
    }

    // Can add 3 and 4 bit widths
    #[test]
    fn test_add_3_4() {
        type A = Sum<W3, W4>;
        assert_eq!(A::BITS, 7);
    }

    // Can subtract 7 from 42
    #[test]
    fn test_sub_42_7() {
        type A = Diff<W42, W7>;
        assert_eq!(A::BITS, 35);
    }

    // Min of 3 and 4 is 3
    #[test]
    fn test_min_3_4() {
        type A = Minimum<W3, W4>;
        assert_eq!(A::BITS, 3);
    }

    // Log2 of 8 is 3
    #[test]
    fn test_log2_of_8() {
        type A = Log2<W8>;
        assert_eq!(A::BITS, 3);
    }

    #[test]
    fn test_max_3_4() {
        type A = Maximum<W3, W4>;
        assert_eq!(A::BITS, 4);
    }
}
