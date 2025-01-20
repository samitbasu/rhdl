use super::cmp::{Cmp, CmpOut, ComparisonResult, Equal, Greater, Less};

pub trait Select<B, C: ComparisonResult> {
    type Output;
}

pub type SelectOut<A, B, C> = <A as Select<B, C>>::Output;

pub trait Max<Rhs = Self> {
    type Output;
}

pub type Maximum<A, B> = <A as Max<B>>::Output;

pub trait Min<Rhs = Self> {
    type Output;
}

pub type Minimum<A, B> = <A as Min<B>>::Output;

impl<A, B> Max<B> for A
where
    A: Cmp<B> + Select<B, CmpOut<A, B>>,
    CmpOut<A, B>: ComparisonResult,
{
    type Output = SelectOut<A, B, CmpOut<A, B>>;
}

impl<A, B> Select<B, Greater> for A {
    type Output = A;
}

impl<A, B> Select<B, Less> for A {
    type Output = B;
}

impl<A, B> Select<B, Equal> for A {
    type Output = A;
}

impl<A, B> Min<B> for A
where
    A: Cmp<B>,
    B: Select<A, CmpOut<A, B>>,
    CmpOut<A, B>: ComparisonResult,
{
    type Output = SelectOut<B, A, CmpOut<A, B>>;
}
