use crate::{
    invert::{Invert, InvertOut},
    trim::{Trim, Trimmed},
    Unsigned,
};

pub trait Normalize {
    type Output: Unsigned;
}

pub type Normalized<A> = <A as Normalize>::Output;

impl<T> Normalize for T
where
    T: Invert,
    InvertOut<T>: Trim,
    Trimmed<InvertOut<T>>: Invert,
{
    type Output = InvertOut<Trimmed<InvertOut<T>>>;
}
