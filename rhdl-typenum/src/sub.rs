use std::ops::Sub;

use seq_macro::seq;

use crate::digits::*;
use crate::operators::Trimmed;
use crate::traits::Trim;
use crate::Diff;
use crate::Sub1;
use crate::{Digit, UInt, UTerm, Unsigned};

impl Sub<UTerm> for UTerm {
    type Output = UTerm;
    fn sub(self, _: UTerm) -> Self::Output {
        UTerm
    }
}

impl Sub<D0> for UTerm {
    type Output = UTerm;
    fn sub(self, _: D0) -> Self::Output {
        UTerm
    }
}

seq!(N in 0..=9 {
    impl Sub<UTerm> for D~N {
        type Output = D~N;
        fn sub(self, _: UTerm) -> Self::Output {
            self
        }
    }
});

impl<U: Unsigned, B: Digit> Sub<D0> for UInt<U, B> {
    type Output = UInt<U, B>;
    fn sub(self, _: D0) -> Self::Output {
        self
    }
}

// Define a macro that generates the sub impl if there is no borrow
macro_rules! sub_impl_no_borrow {
    ($a:ty,$b:ty,$c:ty) => {
        impl<U: Unsigned> Sub<$b> for UInt<U, $a> {
            type Output = UInt<U, $c>;
            fn sub(self, _: $b) -> Self::Output {
                UInt::new()
            }
        }
    };
}

// Define a macro that generates the sub impl with a borrow
macro_rules! sub_impl_with_borrow {
    ($a:ty,$b:ty,$c:ty) => {
        impl<U: Unsigned> Sub<$b> for UInt<U, $a>
        where
            U: Sub<D1>,
            Sub1<U>: Unsigned,
        {
            type Output = UInt<Sub1<U>, $c>;
            fn sub(self, _: $b) -> Self::Output {
                UInt::new()
            }
        }
    };
}

sub_impl_no_borrow!(D1, D1, D0);
sub_impl_no_borrow!(D2, D1, D1);
sub_impl_no_borrow!(D3, D1, D2);
sub_impl_no_borrow!(D4, D1, D3);
sub_impl_no_borrow!(D5, D1, D4);
sub_impl_no_borrow!(D6, D1, D5);
sub_impl_no_borrow!(D7, D1, D6);
sub_impl_no_borrow!(D8, D1, D7);
sub_impl_no_borrow!(D9, D1, D8);
sub_impl_no_borrow!(D2, D2, D0);
sub_impl_no_borrow!(D3, D2, D1);
sub_impl_no_borrow!(D4, D2, D2);
sub_impl_no_borrow!(D5, D2, D3);
sub_impl_no_borrow!(D6, D2, D4);
sub_impl_no_borrow!(D7, D2, D5);
sub_impl_no_borrow!(D8, D2, D6);
sub_impl_no_borrow!(D9, D2, D7);
sub_impl_no_borrow!(D3, D3, D0);
sub_impl_no_borrow!(D4, D3, D1);
sub_impl_no_borrow!(D5, D3, D2);
sub_impl_no_borrow!(D6, D3, D3);
sub_impl_no_borrow!(D7, D3, D4);
sub_impl_no_borrow!(D8, D3, D5);
sub_impl_no_borrow!(D9, D3, D6);
sub_impl_no_borrow!(D4, D4, D0);
sub_impl_no_borrow!(D5, D4, D1);
sub_impl_no_borrow!(D6, D4, D2);
sub_impl_no_borrow!(D7, D4, D3);
sub_impl_no_borrow!(D8, D4, D4);
sub_impl_no_borrow!(D9, D4, D5);
sub_impl_no_borrow!(D5, D5, D0);
sub_impl_no_borrow!(D6, D5, D1);
sub_impl_no_borrow!(D7, D5, D2);
sub_impl_no_borrow!(D8, D5, D3);
sub_impl_no_borrow!(D9, D5, D4);
sub_impl_no_borrow!(D6, D6, D0);
sub_impl_no_borrow!(D7, D6, D1);
sub_impl_no_borrow!(D8, D6, D2);
sub_impl_no_borrow!(D9, D6, D3);
sub_impl_no_borrow!(D7, D7, D0);
sub_impl_no_borrow!(D8, D7, D1);
sub_impl_no_borrow!(D9, D7, D2);
sub_impl_no_borrow!(D8, D8, D0);
sub_impl_no_borrow!(D9, D8, D1);
sub_impl_no_borrow!(D9, D9, D0);

sub_impl_with_borrow!(D0, D1, D9);
sub_impl_with_borrow!(D0, D2, D8);
sub_impl_with_borrow!(D1, D2, D9);
sub_impl_with_borrow!(D0, D3, D7);
sub_impl_with_borrow!(D1, D3, D8);
sub_impl_with_borrow!(D2, D3, D9);
sub_impl_with_borrow!(D0, D4, D6);
sub_impl_with_borrow!(D1, D4, D7);
sub_impl_with_borrow!(D2, D4, D8);
sub_impl_with_borrow!(D3, D4, D9);
sub_impl_with_borrow!(D0, D5, D5);
sub_impl_with_borrow!(D1, D5, D6);
sub_impl_with_borrow!(D2, D5, D7);
sub_impl_with_borrow!(D3, D5, D8);
sub_impl_with_borrow!(D4, D5, D9);
sub_impl_with_borrow!(D0, D6, D4);
sub_impl_with_borrow!(D1, D6, D5);
sub_impl_with_borrow!(D2, D6, D6);
sub_impl_with_borrow!(D3, D6, D7);
sub_impl_with_borrow!(D4, D6, D8);
sub_impl_with_borrow!(D0, D7, D3);
sub_impl_with_borrow!(D1, D7, D4);
sub_impl_with_borrow!(D2, D7, D5);
sub_impl_with_borrow!(D3, D7, D6);
sub_impl_with_borrow!(D4, D7, D7);
sub_impl_with_borrow!(D0, D8, D2);
sub_impl_with_borrow!(D1, D8, D3);
sub_impl_with_borrow!(D2, D8, D4);
sub_impl_with_borrow!(D3, D8, D5);
sub_impl_with_borrow!(D4, D8, D6);
sub_impl_with_borrow!(D0, D9, D1);
sub_impl_with_borrow!(D1, D9, D2);
sub_impl_with_borrow!(D2, D9, D3);
sub_impl_with_borrow!(D3, D9, D4);
sub_impl_with_borrow!(D4, D9, D5);
sub_impl_with_borrow!(D5, D9, D6);
sub_impl_with_borrow!(D6, D9, D7);
sub_impl_with_borrow!(D7, D9, D8);
sub_impl_with_borrow!(D8, D9, D9);

// -- Subtracting unsigned integers
impl<U: Unsigned, B: Digit> Sub<UTerm> for UInt<U, B> {
    type Output = UInt<U, B>;
    fn sub(self, _: UTerm) -> Self::Output {
        self
    }
}

macro_rules! sub_uint_impl_with_no_borrow {
    ($a:ty, $b:ty, $c:ty) => {
        impl<Ul: Unsigned, Ur: Unsigned> Sub<UInt<Ur, $b>> for UInt<Ul, $a>
        where
            Ul: Sub<Ur>,
            UInt<Diff<Ul, Ur>, $c>: Trim,
            Trimmed<UInt<Diff<Ul, Ur>, $c>>: Unsigned,
        {
            type Output = Trimmed<UInt<Diff<Ul, Ur>, $c>>;
            fn sub(self, rhs: UInt<Ur, $b>) -> Self::Output {
                UInt {
                    msb: self.msb - rhs.msb,
                    lsb: <$c>::new(),
                }
                .trim()
            }
        }
    };
}

macro_rules! sub_uint_impl_with_borrow {
    ($a:ty, $b:ty, $c: ty) => {
        impl<Ul: Unsigned, Ur: Unsigned> Sub<UInt<Ur, $b>> for UInt<Ul, $a>
        where
            Ul: Sub<Ur>,
            Diff<Ul, Ur>: Sub<D1>,
        {
            type Output = UInt<Sub1<Diff<Ul, Ur>>, $c>;
            fn sub(self, rhs: UInt<Ur, $b>) -> Self::Output {
                UInt {
                    msb: self.msb - rhs.msb - D1,
                    lsb: <$c>::new(),
                }
            }
        }
    };
}

sub_uint_impl_with_no_borrow!(D0, D0, D0);
sub_uint_impl_with_no_borrow!(D1, D0, D1);
sub_uint_impl_with_no_borrow!(D2, D0, D2);
sub_uint_impl_with_no_borrow!(D3, D0, D3);
sub_uint_impl_with_no_borrow!(D4, D0, D4);
sub_uint_impl_with_no_borrow!(D5, D0, D5);
sub_uint_impl_with_no_borrow!(D6, D0, D6);
sub_uint_impl_with_no_borrow!(D7, D0, D7);
sub_uint_impl_with_no_borrow!(D8, D0, D8);
sub_uint_impl_with_no_borrow!(D9, D0, D9);
sub_uint_impl_with_no_borrow!(D1, D1, D0);
sub_uint_impl_with_no_borrow!(D2, D1, D1);
sub_uint_impl_with_no_borrow!(D3, D1, D2);
sub_uint_impl_with_no_borrow!(D4, D1, D3);
sub_uint_impl_with_no_borrow!(D5, D1, D4);
sub_uint_impl_with_no_borrow!(D6, D1, D5);
sub_uint_impl_with_no_borrow!(D7, D1, D6);
sub_uint_impl_with_no_borrow!(D8, D1, D7);
sub_uint_impl_with_no_borrow!(D9, D1, D8);
sub_uint_impl_with_no_borrow!(D2, D2, D0);
sub_uint_impl_with_no_borrow!(D3, D2, D1);
sub_uint_impl_with_no_borrow!(D4, D2, D2);
sub_uint_impl_with_no_borrow!(D5, D2, D3);
sub_uint_impl_with_no_borrow!(D6, D2, D4);
sub_uint_impl_with_no_borrow!(D7, D2, D5);
sub_uint_impl_with_no_borrow!(D8, D2, D6);
sub_uint_impl_with_no_borrow!(D9, D2, D7);
sub_uint_impl_with_no_borrow!(D3, D3, D0);
sub_uint_impl_with_no_borrow!(D4, D3, D1);
sub_uint_impl_with_no_borrow!(D5, D3, D2);
sub_uint_impl_with_no_borrow!(D6, D3, D3);
sub_uint_impl_with_no_borrow!(D7, D3, D4);
sub_uint_impl_with_no_borrow!(D8, D3, D5);
sub_uint_impl_with_no_borrow!(D9, D3, D6);
sub_uint_impl_with_no_borrow!(D4, D4, D0);
sub_uint_impl_with_no_borrow!(D5, D4, D1);
sub_uint_impl_with_no_borrow!(D6, D4, D2);
sub_uint_impl_with_no_borrow!(D7, D4, D3);
sub_uint_impl_with_no_borrow!(D8, D4, D4);
sub_uint_impl_with_no_borrow!(D9, D4, D5);
sub_uint_impl_with_no_borrow!(D5, D5, D0);
sub_uint_impl_with_no_borrow!(D6, D5, D1);
sub_uint_impl_with_no_borrow!(D7, D5, D2);
sub_uint_impl_with_no_borrow!(D8, D5, D3);
sub_uint_impl_with_no_borrow!(D9, D5, D4);
sub_uint_impl_with_no_borrow!(D6, D6, D0);
sub_uint_impl_with_no_borrow!(D7, D6, D1);
sub_uint_impl_with_no_borrow!(D8, D6, D2);
sub_uint_impl_with_no_borrow!(D9, D6, D3);
sub_uint_impl_with_no_borrow!(D7, D7, D0);
sub_uint_impl_with_no_borrow!(D8, D7, D1);
sub_uint_impl_with_no_borrow!(D9, D7, D2);
sub_uint_impl_with_no_borrow!(D8, D8, D0);
sub_uint_impl_with_no_borrow!(D9, D8, D1);
sub_uint_impl_with_no_borrow!(D9, D9, D0);

sub_uint_impl_with_borrow!(D0, D1, D9);
sub_uint_impl_with_borrow!(D0, D2, D8);
sub_uint_impl_with_borrow!(D1, D2, D9);
sub_uint_impl_with_borrow!(D0, D3, D7);
sub_uint_impl_with_borrow!(D1, D3, D8);
sub_uint_impl_with_borrow!(D2, D3, D9);
sub_uint_impl_with_borrow!(D0, D4, D6);
sub_uint_impl_with_borrow!(D1, D4, D7);
sub_uint_impl_with_borrow!(D2, D4, D8);
sub_uint_impl_with_borrow!(D3, D4, D9);
sub_uint_impl_with_borrow!(D0, D5, D5);
sub_uint_impl_with_borrow!(D1, D5, D6);
sub_uint_impl_with_borrow!(D2, D5, D7);
sub_uint_impl_with_borrow!(D3, D5, D8);
sub_uint_impl_with_borrow!(D4, D5, D9);
sub_uint_impl_with_borrow!(D0, D6, D4);
sub_uint_impl_with_borrow!(D1, D6, D5);
sub_uint_impl_with_borrow!(D2, D6, D6);
sub_uint_impl_with_borrow!(D3, D6, D7);
sub_uint_impl_with_borrow!(D4, D6, D8);
sub_uint_impl_with_borrow!(D0, D7, D3);
sub_uint_impl_with_borrow!(D1, D7, D4);
sub_uint_impl_with_borrow!(D2, D7, D5);
sub_uint_impl_with_borrow!(D3, D7, D6);
sub_uint_impl_with_borrow!(D4, D7, D7);
sub_uint_impl_with_borrow!(D0, D8, D2);
sub_uint_impl_with_borrow!(D1, D8, D3);
sub_uint_impl_with_borrow!(D2, D8, D4);
sub_uint_impl_with_borrow!(D3, D8, D5);
sub_uint_impl_with_borrow!(D4, D8, D6);
sub_uint_impl_with_borrow!(D0, D9, D1);
sub_uint_impl_with_borrow!(D1, D9, D2);
sub_uint_impl_with_borrow!(D2, D9, D3);
sub_uint_impl_with_borrow!(D3, D9, D4);
sub_uint_impl_with_borrow!(D4, D9, D5);
sub_uint_impl_with_borrow!(D5, D9, D6);
sub_uint_impl_with_borrow!(D6, D9, D7);
sub_uint_impl_with_borrow!(D7, D9, D8);
sub_uint_impl_with_borrow!(D8, D9, D9);
