use crate::{digits::*, unsigned::Unsigned};
use std::ops::Add;

use seq_macro::seq;

use crate::unsigned::{T_, U_};

impl Add<D0> for T_ {
    type Output = T_;
    fn add(self, _: D0) -> Self::Output {
        T_
    }
}

seq!(N in 1..=9 {
    impl Add<D~N> for T_ {
        type Output = U_<T_, D~N>;
        fn add(self, _: D~N) -> Self::Output {
            U_::new()
        }
    }
});

// Create a macro by example that takes a list of the form:
// [
//   (D1, D1, D2, D0),
//   (D7, D3, D0, D1),
//  ...]
// And generates a list of AddDigit impls like
// impl AddDigit<D1> for D1 {
//     type Carry = D0;
//     type Output = D2;
// }
macro_rules! add_digit_impl {
    ($a:ty, $b:ty, $c:ty, $d:ty) => {
        impl AddDigit<$a> for $b {
            type Carry = $d;
            type Output = $c;
        }
    };
}

macro_rules! add_digit_impls {
    ( $( ($a:ty, $b:ty, $c:ty, $d:ty) ),* ) => {
        $(
            add_digit_impl!($a, $b, $c, $d);
        )*
    };
}

// OK!  RustC learns to add digits.
add_digit_impls!(
    (D0, D0, D0, D0),
    (D0, D1, D1, D0),
    (D0, D2, D2, D0),
    (D0, D3, D3, D0),
    (D0, D4, D4, D0),
    (D0, D5, D5, D0),
    (D0, D6, D6, D0),
    (D0, D7, D7, D0),
    (D0, D8, D8, D0),
    (D0, D9, D9, D0),
    (D1, D0, D1, D0),
    (D1, D1, D2, D0),
    (D1, D2, D3, D0),
    (D1, D3, D4, D0),
    (D1, D4, D5, D0),
    (D1, D5, D6, D0),
    (D1, D6, D7, D0),
    (D1, D7, D8, D0),
    (D1, D8, D9, D0),
    (D1, D9, D0, D1),
    (D2, D0, D2, D0),
    (D2, D1, D3, D0),
    (D2, D2, D4, D0),
    (D2, D3, D5, D0),
    (D2, D4, D6, D0),
    (D2, D5, D7, D0),
    (D2, D6, D8, D0),
    (D2, D7, D9, D0),
    (D2, D8, D0, D1),
    (D2, D9, D1, D1),
    (D3, D0, D3, D0),
    (D3, D1, D4, D0),
    (D3, D2, D5, D0),
    (D3, D3, D6, D0),
    (D3, D4, D7, D0),
    (D3, D5, D8, D0),
    (D3, D6, D9, D0),
    (D3, D7, D0, D1),
    (D3, D8, D1, D1),
    (D3, D9, D2, D1),
    (D4, D0, D4, D0),
    (D4, D1, D5, D0),
    (D4, D2, D6, D0),
    (D4, D3, D7, D0),
    (D4, D4, D8, D0),
    (D4, D5, D9, D0),
    (D4, D6, D0, D1),
    (D4, D7, D1, D1),
    (D4, D8, D2, D1),
    (D4, D9, D3, D1),
    (D5, D0, D5, D0),
    (D5, D1, D6, D0),
    (D5, D2, D7, D0),
    (D5, D3, D8, D0),
    (D5, D4, D9, D0),
    (D5, D5, D0, D1),
    (D5, D6, D1, D1),
    (D5, D7, D2, D1),
    (D5, D8, D3, D1),
    (D5, D9, D4, D1),
    (D6, D0, D6, D0),
    (D6, D1, D7, D0),
    (D6, D2, D8, D0),
    (D6, D3, D9, D0),
    (D6, D4, D0, D1),
    (D6, D5, D1, D1),
    (D6, D6, D2, D1),
    (D6, D7, D3, D1),
    (D6, D8, D4, D1),
    (D6, D9, D5, D1),
    (D7, D0, D7, D0),
    (D7, D1, D8, D0),
    (D7, D2, D9, D0),
    (D7, D3, D0, D1),
    (D7, D4, D1, D1),
    (D7, D5, D2, D1),
    (D7, D6, D3, D1),
    (D7, D7, D4, D1),
    (D7, D8, D5, D1),
    (D7, D9, D6, D1),
    (D8, D0, D8, D0),
    (D8, D1, D9, D0),
    (D8, D2, D0, D1),
    (D8, D3, D1, D1),
    (D8, D4, D2, D1),
    (D8, D5, D3, D1),
    (D8, D6, D4, D1),
    (D8, D7, D5, D1),
    (D8, D8, D6, D1),
    (D8, D9, D7, D1),
    (D9, D0, D9, D0),
    (D9, D1, D0, D1),
    (D9, D2, D1, D1),
    (D9, D3, D2, D1),
    (D9, D4, D3, D1),
    (D9, D5, D4, D1),
    (D9, D6, D5, D1),
    (D9, D7, D6, D1),
    (D9, D8, D7, D1),
    (D9, D9, D8, D1)
);

pub trait AddDigit<Rhs = Self> {
    type Carry: Digit;
    type Output: Digit;
}

impl<U: Unsigned, B: Digit, A: Digit> Add<A> for U_<U, B>
where
    B: AddDigit<A>,
    U: Add<B::Carry>,
    Sum<U, B::Carry>: Unsigned,
{
    type Output = U_<Sum<U, B::Carry>, <B as AddDigit<A>>::Output>;
    fn add(self, _: A) -> Self::Output {
        U_::new()
    }
}

impl<Ul: Unsigned, Bl: Digit, Ur: Unsigned, Br: Digit> Add<U_<Ur, Br>> for U_<Ul, Bl>
where
    Bl: AddDigit<Br>,
    Ul: Add<Ur>,
    Sum<Ul, Ur>: Unsigned,
    Sum<Ul, Ur>: Add<Bl::Carry>,
    Sum<Sum<Ul, Ur>, Bl::Carry>: Unsigned,
{
    type Output = U_<Sum<Sum<Ul, Ur>, Bl::Carry>, <Bl as AddDigit<Br>>::Output>;
    fn add(self, _: U_<Ur, Br>) -> Self::Output {
        U_::new()
    }
}
// -- Adding unsigned integers

/// UTerm + U = U
impl<U: Unsigned> Add<U> for T_ {
    type Output = U;
    fn add(self, rhs: U) -> Self::Output {
        rhs
    }
}

/// UInt<U,B> + UTerm = UInt<U, B>
impl<U: Unsigned, B: Digit> Add<T_> for U_<U, B> {
    type Output = U_<U, B>;
    fn add(self, _: T_) -> Self::Output {
        self
    }
}

pub type Add1<A> = <A as Add<D1>>::Output;
pub type Sum<A, B> = <A as Add<B>>::Output;
