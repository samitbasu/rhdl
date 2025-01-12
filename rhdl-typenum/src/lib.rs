// Tell clippy to ignore this module
#![allow(clippy::all)]
use rhdl_macro::{add_impl, log2_impl, max_impl, min_impl, sub_impl};
use seq_macro::seq;

pub trait BitWidth: Copy + Clone + PartialEq + Eq + Default + 'static {
    const BITS: usize;
}

pub trait Min<Rhs = Self>
where
    Self: BitWidth,
    Rhs: BitWidth,
{
    /// The type of the minimum of `Self` and `Rhs`
    type Output: BitWidth;
    /// Method returning the minimum
    fn min(self, rhs: Rhs) -> Self::Output;
}

pub trait Max<Rhs = Self>
where
    Self: BitWidth,
    Rhs: BitWidth,
{
    /// The type of the maximum of `Self` and `Rhs`
    type Output: BitWidth;
    /// Method returning the maximum
    fn max(self, rhs: Rhs) -> Self::Output;
}

/// Alias for the associated type of `Min`: `Minimum<A, B> = <A as Min<B>>::Output`
pub type Minimum<A, B> = <A as Min<B>>::Output;

/// Alias for the associated type of `Max`: `Maximum<A, B> = <A as Max<B>>::Output`
pub type Maximum<A, B> = <A as Max<B>>::Output;

pub type Sum<A, B> = <A as std::ops::Add<B>>::Output;

// Re-export the typenum types so that users of this crate
// don't have to import them separately.
pub use typenum::{Diff, Log2, Logarithm2, Unsigned};

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

        add_impl!(N);
        sub_impl!(N);
        max_impl!(N);
        min_impl!(N);
        log2_impl!(N);
    )*
});

impl<W> Max<W> for W
where
    W: BitWidth,
{
    type Output = W;
    fn max(self, _rhs: W) -> Self::Output {
        self
    }
}

impl<W> Min<W> for W
where
    W: BitWidth,
{
    type Output = W;
    fn min(self, _rhs: W) -> Self::Output {
        self
    }
}

pub trait ToBitWidth {
    type Output: BitWidth;
}

pub struct Const<const N: usize>;

seq!(N in 1..=128 {
    #(
    impl ToBitWidth for Const<N> {
        type Output = W~N;
    })*
});

pub type WN<const N: usize> = <Const<N> as ToBitWidth>::Output;

#[cfg(test)]
mod tests {
    use typenum::{Diff, Log2, Sum, Unsigned};

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
        type B = Log2<W9>;
        assert_eq!(B::BITS, 4);
    }

    #[test]
    fn test_max_3_4() {
        type A = Maximum<W3, W4>;
        assert_eq!(A::BITS, 4);
    }
}
