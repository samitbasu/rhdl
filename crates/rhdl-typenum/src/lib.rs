//! Type-level unsigned integers and operations for RHDL
#![allow(clippy::all)]
#![warn(missing_docs)]
/// Module to provide interop with Rust's const generics
pub mod const_generics;
/// Type-level unsigned integers of U1 to U128
pub mod consts;
/// Common imports for using rhdl-typenum
pub mod prelude;
/// Type-level unsigned integers trait and implementations
pub mod unsigned;

use prelude::Unsigned;
/// Re-export the `op` macro for defining operations on type-level integers
pub use rhdl_macro::op;

use seq_macro::seq;

use consts::*;

/// Marker trait for type-level unsigned integers comparison
/// Implementations of this trait are generated automatically
/// for type pairs where the comparison holds true.
pub trait IsLessThan<Rhs> {}

/// Marker trait for type-level unsigned integers comparison
/// Implementations of this trait are generated automatically
/// for type pairs where the comparison holds true.
pub trait IsGreaterThan<Rhs> {}

/// Marker trait for type-level unsigned integers equality
/// Implementations of this trait are generated automatically
/// for type pairs where the equality holds true.
pub trait IsEqualTo<Rhs> {}

/// Marker trait for type-level unsigned integers comparison
/// Implementations of this trait are generated automatically
/// for type pairs where the comparison holds true.
pub trait IsLessThanOrEqualTo<Rhs> {}

/// Marker trait for type-level unsigned integers comparison
/// Implementations of this trait are generated automatically
/// for type pairs where the comparison holds true.
pub trait IsGreaterThanOrEqualTo<Rhs> {}

seq!(N in 1..=128 {
    impl IsEqualTo<U~N> for U~N {}
});

impl<A, B> IsLessThanOrEqualTo<B> for A
where
    A: IsLessThan<B>,
    B: IsGreaterThan<A>,
{
}

impl<A, B> IsGreaterThanOrEqualTo<B> for A
where
    A: IsGreaterThan<B>,
    B: IsLessThan<A>,
{
}

impl<A, B> IsGreaterThan<B> for A where B: IsLessThan<A> {}

/// Trait operator to compute the maximum of two type-level unsigned integers.
pub trait Max<Rhs = Self> {
    /// The output type after computing the maximum.
    type Output;

    /// Compute the maximum of two type-level unsigned integers.
    fn max(self, rhs: Rhs) -> Self::Output;
}

impl<A> Max for A
where
    A: Unsigned,
{
    type Output = A;

    fn max(self, _: A) -> Self::Output {
        self
    }
}

/// Trait operator to compute the minimum of two type-level unsigned integers.
pub trait Min<Rhs = Self> {
    /// The output type after computing the minimum.
    type Output;

    /// Compute the minimum of two type-level unsigned integers.
    fn min(self, rhs: Rhs) -> Self::Output;
}

impl<A> Min for A
where
    A: Unsigned,
{
    type Output = A;

    fn min(self, _: A) -> Self::Output {
        self
    }
}

/// Type alias for adding 1 to a type-level unsigned integer.
pub type Add1<A> = <A as std::ops::Add<U1>>::Output;
/// Type alias for summing two type-level unsigned integers.
pub type Sum<A, B> = <A as std::ops::Add<B>>::Output;
/// Type alias for subtracting one type-level unsigned integer from another.
pub type Diff<A, B> = <A as std::ops::Sub<B>>::Output;
/// Type alias for computing the maximum of two type-level unsigned integers.
pub type Maximum<A, B> = <A as Max<B>>::Output;
/// Type alias for computing the minimum of two type-level unsigned integers.
pub type Minimum<A, B> = <A as Min<B>>::Output;

include!(concat!(env!("OUT_DIR"), "/typenum_add_impls.rs"));
include!(concat!(env!("OUT_DIR"), "/typenum_sub_impls.rs"));
include!(concat!(env!("OUT_DIR"), "/typenum_max_impls.rs"));
include!(concat!(env!("OUT_DIR"), "/typenum_min_impls.rs"));

#[cfg(test)]
mod tests {
    // There are not a lot of tests here, as the tables are all automatically
    // generated.
    use super::*;
    use static_assertions::assert_impl_all;

    #[test]
    fn test_add() {
        let a = U32;
        let b = U32;
        let c = a + b;
        let _d = c - b;
        type _D = Sum<U32, U32>;
        assert_impl_all!(_D: IsEqualTo<U64>);
    }

    #[test]
    fn test_sub() {
        type _D = Diff<U34, U2>;
        assert_impl_all!(_D: IsEqualTo<U32>);
    }

    #[test]
    fn test_max() {
        type _D = Maximum<U34, U15>;
        assert_impl_all!(_D: IsEqualTo<U34>);
    }

    #[test]
    fn test_min() {
        type _D = Minimum<U34, U15>;
        assert_impl_all!(_D: IsEqualTo<U15>);
    }
}
