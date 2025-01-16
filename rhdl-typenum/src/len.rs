use std::ops::Add;

use crate::{
    digits::D1,
    operators::{Add1, Length},
    traits::{Digit, Len, Unsigned},
    unsigned::UInt,
};

impl<U: Unsigned, B: Digit> Len for UInt<U, B>
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
