use crate::{
    digits::Digit,
    unsigned::{Unsigned, T_, U_},
};

// Reverse the order of digits, so that the MSB is now the LSB.
pub trait Invert {
    type Output: Unsigned;
}

pub type InvertOut<A> = <A as Invert>::Output;

// The start conditions are unique for inversion.  Only the first
// (right most or LSB) is terminated.  So we start with that
// and then delegate the rest to the private inversion trait.
impl Invert for T_ {
    type Output = T_;
}

impl<U: Unsigned, B: Digit> Invert for U_<U, B>
where
    U: PrivateInvert<U_<T_, B>>,
{
    type Output = PrivateInvertOut<U, U_<T_, B>>;
}

pub trait PrivateInvert<SoFar> {
    type Output: Unsigned;
}

type PrivateInvertOut<A, SoFar> = <A as PrivateInvert<SoFar>>::Output;

impl<SoFar> PrivateInvert<SoFar> for T_
where
    SoFar: Unsigned,
{
    type Output = SoFar;
}

impl<U: Unsigned, B: Digit, SoFar: Unsigned> PrivateInvert<SoFar> for U_<U, B>
where
    U: PrivateInvert<U_<SoFar, B>>,
{
    type Output = PrivateInvertOut<U, U_<SoFar, B>>;
}
