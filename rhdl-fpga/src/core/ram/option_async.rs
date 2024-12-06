use rhdl::prelude::*;

#[derive(Debug, Clone, Default, Circuit, CircuitDQ)]
pub struct U<T: Digital + Default, W: Domain, R: Domain, const N: usize> {
    inner: super::asynchronous::U<T, W, R, N>,
}

impl<T: Digital + Default, W: Domain, R: Domain, const N: usize> U<T, W, R, N> {
    pub fn new(initial: impl IntoIterator<Item = (Bits<N>, T)>) -> Self {
        Self {
            inner: super::asynchronous::U::new(initial),
        }
    }
}
