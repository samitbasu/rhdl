use std::ops::Add;

use crate::{
    add::Add1,
    consts::U0,
    digits::{Digit, D1},
    unsigned::{Unsigned, T_, U_},
};

pub trait Len {
    type Output: Unsigned;
    fn len(&self) -> Self::Output;
}

pub type Length<T> = <T as Len>::Output;

impl<U: Unsigned, B: Digit> Len for U_<U, B>
where
    U: Len,
    Length<U>: Add<D1>,
    Add1<Length<U>>: Unsigned,
{
    type Output = Add1<Length<U>>;
    fn len(&self) -> Self::Output {
        self.msb.len() + D1
    }
}

impl Len for T_ {
    type Output = U0;
    fn len(&self) -> Self::Output {
        T_
    }
}
