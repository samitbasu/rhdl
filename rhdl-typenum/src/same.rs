use crate::{bools::True, Bool};

pub trait IsSame<Rhs = Self> {
    type Output: Bool;
}

pub type IsSameAs<A, B> = <A as IsSame<B>>::Output;

impl<A> IsSame for A {
    type Output = True;
}
