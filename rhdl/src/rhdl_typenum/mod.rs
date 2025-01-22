// Tell clippy to ignore this module
#![allow(clippy::all)]
pub mod const_generics;
pub mod consts;
pub mod prelude;
pub mod unsigned;
use prelude::Unsigned;
pub use rhdl_macro::op;

use seq_macro::seq;

use consts::*;

pub trait IsLessThan<Rhs> {}

pub trait IsGreaterThan<Rhs> {}

pub trait IsEqualTo<Rhs> {}

pub trait IsLessThanOrEqualTo<Rhs> {}

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

pub trait Max<Rhs = Self> {
    type Output;

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

pub trait Min<Rhs = Self> {
    type Output;

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

pub type Add1<A> = <A as std::ops::Add<U1>>::Output;
pub type Sum<A, B> = <A as std::ops::Add<B>>::Output;
pub type Diff<A, B> = <A as std::ops::Sub<B>>::Output;
pub type Maximum<A, B> = <A as Max<B>>::Output;
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
        let d = c - b;
        type D = Sum<U32, U32>;
        assert_impl_all!(D: IsEqualTo<U64>);
    }

    #[test]
    fn test_sub() {
        type D = Diff<U34, U2>;
        assert_impl_all!(D: IsEqualTo<U32>);
    }

    #[test]
    fn test_max() {
        type D = Maximum<U34, U15>;
        assert_impl_all!(D: IsEqualTo<U34>);
    }

    #[test]
    fn test_min() {
        type D = Minimum<U34, U15>;
        assert_impl_all!(D: IsEqualTo<U15>);
    }
}
