use crate::operators::Add1;
use crate::{
    digits::*,
    traits::{Digit, Unsigned},
};
use std::ops::Add;

use seq_macro::seq;

use crate::{
    digits::D0,
    operators::*,
    unsigned::{UInt, UTerm},
};

impl Add<D0> for UTerm {
    type Output = UTerm;
    fn add(self, _: D0) -> Self::Output {
        UTerm
    }
}

seq!(N in 1..=9 {
    impl Add<D~N> for UTerm {
        type Output = UInt<UTerm, D~N>;
        fn add(self, _: D~N) -> Self::Output {
            UInt::new()
        }
    }
});

impl<U: Unsigned, B: Digit> Add<D0> for UInt<U, B> {
    type Output = UInt<U, B>;
    fn add(self, _: D0) -> Self::Output {
        Self::Output::new()
    }
}

seq!(N in 1..=9 {
    impl<U: Unsigned> Add<D~N> for UInt<U, D0> {
        type Output = UInt<U, D~N>;
        fn add(self, _: D~N) -> Self::Output {
            UInt::new()
        }
    }
});

// Define a macro that generates the add impl if there is no carry
macro_rules! add_impl_no_carry {
    ($a:ty,$b:ty,$c:ty) => {
        impl<U: Unsigned> Add<$b> for UInt<U, $a> {
            type Output = UInt<U, $c>;
            fn add(self, _: $b) -> Self::Output {
                UInt::new()
            }
        }
    };
}

// Define a macro that generates the add impl if there is a carry
macro_rules! add_impl_with_carry {
    ($a:ty,$b:ty,$c:ty) => {
        impl<U: Unsigned> Add<$b> for UInt<U, $a>
        where
            U: Add<D1>,
            Add1<U>: Unsigned,
        {
            type Output = UInt<Add1<U>, $c>;
            fn add(self, _: $b) -> Self::Output {
                UInt::new()
            }
        }
    };
}

add_impl_no_carry!(D1, D1, D2);
add_impl_no_carry!(D2, D1, D3);
add_impl_no_carry!(D3, D1, D4);
add_impl_no_carry!(D4, D1, D5);
add_impl_no_carry!(D5, D1, D6);
add_impl_no_carry!(D6, D1, D7);
add_impl_no_carry!(D7, D1, D8);
add_impl_no_carry!(D8, D1, D9);
add_impl_no_carry!(D1, D2, D3);
add_impl_no_carry!(D2, D2, D4);
add_impl_no_carry!(D3, D2, D5);
add_impl_no_carry!(D4, D2, D6);
add_impl_no_carry!(D5, D2, D7);
add_impl_no_carry!(D6, D2, D8);
add_impl_no_carry!(D7, D2, D9);
add_impl_no_carry!(D1, D3, D4);
add_impl_no_carry!(D2, D3, D5);
add_impl_no_carry!(D3, D3, D6);
add_impl_no_carry!(D4, D3, D7);
add_impl_no_carry!(D5, D3, D8);
add_impl_no_carry!(D6, D3, D9);
add_impl_no_carry!(D1, D4, D5);
add_impl_no_carry!(D2, D4, D6);
add_impl_no_carry!(D3, D4, D7);
add_impl_no_carry!(D4, D4, D8);
add_impl_no_carry!(D5, D4, D9);
add_impl_no_carry!(D1, D5, D6);
add_impl_no_carry!(D2, D5, D7);
add_impl_no_carry!(D3, D5, D8);
add_impl_no_carry!(D4, D5, D9);
add_impl_no_carry!(D1, D6, D7);
add_impl_no_carry!(D2, D6, D8);
add_impl_no_carry!(D3, D6, D9);
add_impl_no_carry!(D1, D7, D8);
add_impl_no_carry!(D2, D7, D9);
add_impl_no_carry!(D1, D8, D9);

add_impl_with_carry!(D9, D1, D0);
add_impl_with_carry!(D8, D2, D0);
add_impl_with_carry!(D9, D2, D1);
add_impl_with_carry!(D7, D3, D0);
add_impl_with_carry!(D8, D3, D1);
add_impl_with_carry!(D9, D3, D2);
add_impl_with_carry!(D6, D4, D0);
add_impl_with_carry!(D7, D4, D1);
add_impl_with_carry!(D8, D4, D2);
add_impl_with_carry!(D9, D4, D3);
add_impl_with_carry!(D5, D5, D0);
add_impl_with_carry!(D6, D5, D1);
add_impl_with_carry!(D7, D5, D2);
add_impl_with_carry!(D8, D5, D3);
add_impl_with_carry!(D9, D5, D4);
add_impl_with_carry!(D4, D6, D0);
add_impl_with_carry!(D5, D6, D1);
add_impl_with_carry!(D6, D6, D2);
add_impl_with_carry!(D7, D6, D3);
add_impl_with_carry!(D8, D6, D4);
add_impl_with_carry!(D9, D6, D5);
add_impl_with_carry!(D3, D7, D0);
add_impl_with_carry!(D4, D7, D1);
add_impl_with_carry!(D5, D7, D2);
add_impl_with_carry!(D6, D7, D3);
add_impl_with_carry!(D7, D7, D4);
add_impl_with_carry!(D8, D7, D5);
add_impl_with_carry!(D9, D7, D6);
add_impl_with_carry!(D2, D8, D0);
add_impl_with_carry!(D3, D8, D1);
add_impl_with_carry!(D4, D8, D2);
add_impl_with_carry!(D5, D8, D3);
add_impl_with_carry!(D6, D8, D4);
add_impl_with_carry!(D7, D8, D5);
add_impl_with_carry!(D8, D8, D6);
add_impl_with_carry!(D9, D8, D7);
add_impl_with_carry!(D1, D9, D0);
add_impl_with_carry!(D2, D9, D1);
add_impl_with_carry!(D3, D9, D2);
add_impl_with_carry!(D4, D9, D3);
add_impl_with_carry!(D5, D9, D4);
add_impl_with_carry!(D6, D9, D5);
add_impl_with_carry!(D7, D9, D6);
add_impl_with_carry!(D8, D9, D7);
add_impl_with_carry!(D9, D9, D8);

// -- Adding unsigned integers

/// UTerm + U = U
impl<U: Unsigned> Add<U> for UTerm {
    type Output = U;
    fn add(self, rhs: U) -> Self::Output {
        rhs
    }
}

/// UInt<U,B> + UTerm = UInt<U, B>
impl<U: Unsigned, B: Digit> Add<UTerm> for UInt<U, B> {
    type Output = UInt<U, B>;
    fn add(self, _: UTerm) -> Self::Output {
        self
    }
}

macro_rules! add_uint_impl_with_no_carry {
    ($a:ty,$b:ty,$c:ty) => {
        impl<Ul: Unsigned, Ur: Unsigned> Add<UInt<Ur, $b>> for UInt<Ul, $a>
        where
            Ul: Add<Ur>,
        {
            type Output = UInt<<Ul as Add<Ur>>::Output, $c>;
            fn add(self, rhs: UInt<Ur, $b>) -> Self::Output {
                UInt {
                    msb: self.msb + rhs.msb,
                    lsb: <$c>::new(),
                }
            }
        }
    };
}

macro_rules! add_uint_impl_with_carry {
    ($a:ty,$b:ty,$c:ty) => {
        impl<Ul: Unsigned, Ur: Unsigned> Add<UInt<Ur, $b>> for UInt<Ul, $a>
        where
            Ul: Add<Ur>,
            Sum<Ul, Ur>: Add<D1>,
        {
            type Output = UInt<Add1<Sum<Ul, Ur>>, $c>;
            fn add(self, rhs: UInt<Ur, $b>) -> Self::Output {
                UInt {
                    msb: self.msb + rhs.msb + D1,
                    lsb: <$c>::new(),
                }
            }
        }
    };
}

seq!(N in 0..=9 {
    add_uint_impl_with_no_carry!(D~N, D0, D~N);
});
add_uint_impl_with_no_carry!(D0, D1, D1);
add_uint_impl_with_no_carry!(D1, D1, D2);
add_uint_impl_with_no_carry!(D2, D1, D3);
add_uint_impl_with_no_carry!(D3, D1, D4);
add_uint_impl_with_no_carry!(D4, D1, D5);
add_uint_impl_with_no_carry!(D5, D1, D6);
add_uint_impl_with_no_carry!(D6, D1, D7);
add_uint_impl_with_no_carry!(D7, D1, D8);
add_uint_impl_with_no_carry!(D8, D1, D9);
add_uint_impl_with_no_carry!(D1, D2, D3);
add_uint_impl_with_no_carry!(D2, D2, D4);
add_uint_impl_with_no_carry!(D3, D2, D5);
add_uint_impl_with_no_carry!(D4, D2, D6);
add_uint_impl_with_no_carry!(D5, D2, D7);
add_uint_impl_with_no_carry!(D6, D2, D8);
add_uint_impl_with_no_carry!(D7, D2, D9);
add_uint_impl_with_no_carry!(D1, D3, D4);
add_uint_impl_with_no_carry!(D2, D3, D5);
add_uint_impl_with_no_carry!(D3, D3, D6);
add_uint_impl_with_no_carry!(D4, D3, D7);
add_uint_impl_with_no_carry!(D5, D3, D8);
add_uint_impl_with_no_carry!(D6, D3, D9);
add_uint_impl_with_no_carry!(D1, D4, D5);
add_uint_impl_with_no_carry!(D2, D4, D6);
add_uint_impl_with_no_carry!(D3, D4, D7);
add_uint_impl_with_no_carry!(D4, D4, D8);
add_uint_impl_with_no_carry!(D5, D4, D9);
add_uint_impl_with_no_carry!(D1, D5, D6);
add_uint_impl_with_no_carry!(D2, D5, D7);
add_uint_impl_with_no_carry!(D3, D5, D8);
add_uint_impl_with_no_carry!(D4, D5, D9);
add_uint_impl_with_no_carry!(D1, D6, D7);
add_uint_impl_with_no_carry!(D2, D6, D8);
add_uint_impl_with_no_carry!(D3, D6, D9);
add_uint_impl_with_no_carry!(D1, D7, D8);
add_uint_impl_with_no_carry!(D2, D7, D9);
add_uint_impl_with_no_carry!(D1, D8, D9);

add_uint_impl_with_carry!(D9, D1, D0);
add_uint_impl_with_carry!(D8, D2, D0);
add_uint_impl_with_carry!(D9, D2, D1);
add_uint_impl_with_carry!(D7, D3, D0);
add_uint_impl_with_carry!(D8, D3, D1);
add_uint_impl_with_carry!(D9, D3, D2);
add_uint_impl_with_carry!(D6, D4, D0);
add_uint_impl_with_carry!(D7, D4, D1);
add_uint_impl_with_carry!(D8, D4, D2);
add_uint_impl_with_carry!(D9, D4, D3);
add_uint_impl_with_carry!(D5, D5, D0);
add_uint_impl_with_carry!(D6, D5, D1);
add_uint_impl_with_carry!(D7, D5, D2);
add_uint_impl_with_carry!(D8, D5, D3);
add_uint_impl_with_carry!(D9, D5, D4);
add_uint_impl_with_carry!(D4, D6, D0);
add_uint_impl_with_carry!(D5, D6, D1);
add_uint_impl_with_carry!(D6, D6, D2);
add_uint_impl_with_carry!(D7, D6, D3);
add_uint_impl_with_carry!(D8, D6, D4);
add_uint_impl_with_carry!(D9, D6, D5);
add_uint_impl_with_carry!(D3, D7, D0);
add_uint_impl_with_carry!(D4, D7, D1);
add_uint_impl_with_carry!(D5, D7, D2);
add_uint_impl_with_carry!(D6, D7, D3);
add_uint_impl_with_carry!(D7, D7, D4);
add_uint_impl_with_carry!(D8, D7, D5);
add_uint_impl_with_carry!(D9, D7, D6);
add_uint_impl_with_carry!(D2, D8, D0);
add_uint_impl_with_carry!(D3, D8, D1);
add_uint_impl_with_carry!(D4, D8, D2);
add_uint_impl_with_carry!(D5, D8, D3);
add_uint_impl_with_carry!(D6, D8, D4);
add_uint_impl_with_carry!(D7, D8, D5);
add_uint_impl_with_carry!(D8, D8, D6);
add_uint_impl_with_carry!(D9, D8, D7);
add_uint_impl_with_carry!(D1, D9, D0);
add_uint_impl_with_carry!(D2, D9, D1);
add_uint_impl_with_carry!(D3, D9, D2);
add_uint_impl_with_carry!(D4, D9, D3);
add_uint_impl_with_carry!(D5, D9, D4);
add_uint_impl_with_carry!(D6, D9, D5);
add_uint_impl_with_carry!(D7, D9, D6);
add_uint_impl_with_carry!(D8, D9, D7);
add_uint_impl_with_carry!(D9, D9, D8);
