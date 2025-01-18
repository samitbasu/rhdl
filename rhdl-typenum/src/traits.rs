use crate::cmp::ComparisonResult;

pub trait Len {
    type Output: Unsigned;
    fn len(&self) -> Self::Output;
}

pub trait Bool: Copy + Default + 'static {
    const BOOL: bool;
}

pub trait Digit: Copy + Default + 'static {
    const DIGIT_USIZE: usize = 0;
}

pub trait Unsigned: Copy + Default + 'static {
    const USIZE: usize = 0;
    fn new() -> Self {
        Self::default()
    }
}

pub trait Select<B: Unsigned, C: ComparisonResult> {
    type Output: Unsigned;
}

pub type SelectOut<A, B, C> = <A as Select<B, C>>::Output;

pub trait Max<Rhs = Self> {
    type Output: Unsigned;
}

pub type Maximum<A, B> = <A as Max<B>>::Output;

pub trait Min<Rhs = Self> {
    type Output: Unsigned;
}

pub type Minimum<A, B> = <A as Min<B>>::Output;

pub trait IsLess<Rhs = Self> {
    type Output: Bool;
}

pub type IsLessThan<A, B> = <A as IsLess<B>>::Output;

pub trait IsGreater<Rhs = Self> {
    type Output: Bool;
}

pub type IsGreaterThan<A, B> = <A as IsGreater<B>>::Output;

pub trait IsEqual<Rhs = Self> {
    type Output: Bool;
}

pub type IsEqualTo<A, B> = <A as IsEqual<B>>::Output;

pub trait IsLessThanOrEqual<Rhs = Self> {
    type Output: Bool;
}

pub type IsLessThanOrEqualTo<A, B> = <A as IsLessThanOrEqual<B>>::Output;

pub trait IsGreaterThanOrEqual<Rhs = Self> {
    type Output: Bool;
}

pub type IsGreaterThanOrEqualTo<A, B> = <A as IsGreaterThanOrEqual<B>>::Output;

pub trait IsNotEqual<Rhs = Self> {
    type Output: Bool;
}

pub type IsNotEqualTo<A, B> = <A as IsNotEqual<B>>::Output;
