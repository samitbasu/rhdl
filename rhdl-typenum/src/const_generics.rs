use crate::prelude::Unsigned;

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug, Default)]
pub struct Const<const N: usize>;

impl<const N: usize> Unsigned for Const<N> {
    const USIZE: usize = N;
}
