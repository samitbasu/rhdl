use seq_macro::seq;

use super::digits::*;
use super::unsigned::{Unsigned, T_, U_};

pub trait Trim {
    type Output: Unsigned;
}

pub type Trimmed<A> = <A as Trim>::Output;

// Trim all trailing zeros from a number

impl Trim for T_ {
    type Output = T_;
}

impl<U: Unsigned> Trim for U_<U, D0>
where
    U: Trim,
{
    type Output = Trimmed<U>;
}

seq!(N in 1..=9 {
    impl<U: Unsigned> Trim for U_<U, D~N> {
        type Output = Self;
    }
});