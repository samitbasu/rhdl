use std::ops::Sub;

use seq_macro::seq;

use super::digits::*;
use super::normalize::{Normalize, Normalized};
use super::prelude::Unsigned;
use super::unsigned::{T_, U_};

impl Sub<T_> for T_ {
    type Output = T_;
    fn sub(self, _: T_) -> Self::Output {
        T_
    }
}

impl PrivateSub<T_> for T_ {
    type Output = T_;
}

impl Sub<D0> for T_ {
    type Output = T_;
    fn sub(self, _: D0) -> Self::Output {
        T_
    }
}

impl PrivateSub<D0> for T_ {
    type Output = T_;
}

seq!(N in 0..=9 {
    impl Sub<T_> for D~N {
        type Output = D~N;
        fn sub(self, _: T_) -> Self::Output {
            self
        }
    }
});

pub trait PrivateSub<Rhs> {
    type Output;
}

pub type PDiff<A, B> = <A as PrivateSub<B>>::Output;

pub trait SubDigit<Rhs> {
    type Borrow: Digit;
    type Output: Digit;
}

macro_rules! sub_digit_impl {
    ($a: ty, $b: ty, $c: ty, $d: ty) => {
        impl SubDigit<$b> for $a {
            type Borrow = $d;
            type Output = $c;
        }
    };
}

macro_rules! sub_digit_impls {
    ( $( ($a: ty, $b: ty, $c: ty, $d: ty) ),* ) => {
        $(
            sub_digit_impl!($a, $b, $c, $d);
        )*
    };
}

// OK! RustC learns to subtract digits.
sub_digit_impls!(
    (D0, D0, D0, D0),
    (D0, D1, D9, D1),
    (D0, D2, D8, D1),
    (D0, D3, D7, D1),
    (D0, D4, D6, D1),
    (D0, D5, D5, D1),
    (D0, D6, D4, D1),
    (D0, D7, D3, D1),
    (D0, D8, D2, D1),
    (D0, D9, D1, D1),
    (D1, D0, D1, D0),
    (D1, D1, D0, D0),
    (D1, D2, D9, D1),
    (D1, D3, D8, D1),
    (D1, D4, D7, D1),
    (D1, D5, D6, D1),
    (D1, D6, D5, D1),
    (D1, D7, D4, D1),
    (D1, D8, D3, D1),
    (D1, D9, D2, D1),
    (D2, D0, D2, D0),
    (D2, D1, D1, D0),
    (D2, D2, D0, D0),
    (D2, D3, D9, D1),
    (D2, D4, D8, D1),
    (D2, D5, D7, D1),
    (D2, D6, D6, D1),
    (D2, D7, D5, D1),
    (D2, D8, D4, D1),
    (D2, D9, D3, D1),
    (D3, D0, D3, D0),
    (D3, D1, D2, D0),
    (D3, D2, D1, D0),
    (D3, D3, D0, D0),
    (D3, D4, D9, D1),
    (D3, D5, D8, D1),
    (D3, D6, D7, D1),
    (D3, D7, D6, D1),
    (D3, D8, D5, D1),
    (D3, D9, D4, D1),
    (D4, D0, D4, D0),
    (D4, D1, D3, D0),
    (D4, D2, D2, D0),
    (D4, D3, D1, D0),
    (D4, D4, D0, D0),
    (D4, D5, D9, D1),
    (D4, D6, D8, D1),
    (D4, D7, D7, D1),
    (D4, D8, D6, D1),
    (D4, D9, D5, D1),
    (D5, D0, D5, D0),
    (D5, D1, D4, D0),
    (D5, D2, D3, D0),
    (D5, D3, D2, D0),
    (D5, D4, D1, D0),
    (D5, D5, D0, D0),
    (D5, D6, D9, D1),
    (D5, D7, D8, D1),
    (D5, D8, D7, D1),
    (D5, D9, D6, D1),
    (D6, D0, D6, D0),
    (D6, D1, D5, D0),
    (D6, D2, D4, D0),
    (D6, D3, D3, D0),
    (D6, D4, D2, D0),
    (D6, D5, D1, D0),
    (D6, D6, D0, D0),
    (D6, D7, D9, D1),
    (D6, D8, D8, D1),
    (D6, D9, D7, D1),
    (D7, D0, D7, D0),
    (D7, D1, D6, D0),
    (D7, D2, D5, D0),
    (D7, D3, D4, D0),
    (D7, D4, D3, D0),
    (D7, D5, D2, D0),
    (D7, D6, D1, D0),
    (D7, D7, D0, D0),
    (D7, D8, D9, D1),
    (D7, D9, D8, D1),
    (D8, D0, D8, D0),
    (D8, D1, D7, D0),
    (D8, D2, D6, D0),
    (D8, D3, D5, D0),
    (D8, D4, D4, D0),
    (D8, D5, D3, D0),
    (D8, D6, D2, D0),
    (D8, D7, D1, D0),
    (D8, D8, D0, D0),
    (D8, D9, D9, D1),
    (D9, D0, D9, D0),
    (D9, D1, D8, D0),
    (D9, D2, D7, D0),
    (D9, D3, D6, D0),
    (D9, D4, D5, D0),
    (D9, D5, D4, D0),
    (D9, D6, D3, D0),
    (D9, D7, D2, D0),
    (D9, D8, D1, D0),
    (D9, D9, D0, D0)
);

impl<U: Unsigned, B: Digit, A: Digit> PrivateSub<A> for U_<U, B>
where
    B: SubDigit<A>,
    U: Sub<B::Borrow>,
{
    type Output = U_<Diff<U, B::Borrow>, B::Output>;
}

impl<Ul: Unsigned, Bl: Digit, Ur: Unsigned, Br: Digit> PrivateSub<U_<Ur, Br>> for U_<Ul, Bl>
where
    Bl: SubDigit<Br>,
    Ul: PrivateSub<Ur>,
    PDiff<Ul, Ur>: PrivateSub<Bl::Borrow>,
{
    type Output = U_<PDiff<PDiff<Ul, Ur>, Bl::Borrow>, Bl::Output>;
}

impl<U: Unsigned, B: Digit> PrivateSub<T_> for U_<U, B> {
    type Output = U_<U, B>;
}

impl<U: Unsigned, B: Digit, A: Digit> Sub<A> for U_<U, B>
where
    U_<U, B>: PrivateSub<A>,
    PDiff<U_<U, B>, A>: Normalize,
{
    type Output = Normalized<PDiff<U_<U, B>, A>>;
    fn sub(self, _rhs: A) -> Self::Output {
        Self::Output::new()
    }
}

// -- Subtracting unsigned integers
impl<U: Unsigned, B: Digit> Sub<T_> for U_<U, B> {
    type Output = U_<U, B>;
    fn sub(self, _: T_) -> Self::Output {
        self
    }
}

impl<Ul: Unsigned, Bl: Digit, Ur: Unsigned, Br: Digit> Sub<U_<Ur, Br>> for U_<Ul, Bl>
where
    U_<Ul, Bl>: PrivateSub<U_<Ur, Br>>,
    PDiff<U_<Ul, Bl>, U_<Ur, Br>>: Normalize,
{
    type Output = Normalized<PDiff<U_<Ul, Bl>, U_<Ur, Br>>>;
    fn sub(self, _: U_<Ur, Br>) -> Self::Output {
        Self::Output::new()
    }
}

pub type Sub1<A> = <A as Sub<D1>>::Output;
pub type Diff<A, B> = <A as Sub<B>>::Output;
