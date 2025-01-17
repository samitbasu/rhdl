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

pub trait Trim {
    type Output: Unsigned;
    fn trim(&self) -> Self::Output;
}

pub trait Select<A: Unsigned, B: Unsigned> {
    type Output: Unsigned;
    fn select(&self, _: A, _: B) -> Self::Output;
}

pub trait Cmp<Rhs = Self> {
    /// The result of the comparison. It should only ever be one of `Greater`, `Less`, or `Equal`.
    type Output: ComparisonResult;
}

pub type CmpOut<A, B> = <A as Cmp<B>>::Output;
