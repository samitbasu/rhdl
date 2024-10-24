use rhdl::prelude::*;

use super::constant;
use super::counter;

// A strobe is a pulse that is high for one cycle when the input
// enable signal is high and the counter reaches the threshold.
#[derive(Clone, Debug, Synchronous)]
#[rhdl(kernel=strobe::<{N}>)]
#[rhdl(auto_dq)]
pub struct U<const N: usize> {
    count: counter::U<N>,
    threshold: constant::U<Bits<N>>,
}

impl<const N: usize> U<N> {
    pub fn new(threshold: Bits<N>) -> Self {
        Self {
            count: counter::U::default(),
            threshold: constant::U::new(threshold),
        }
    }
}

impl<const N: usize> SynchronousIO for U<N> {
    type I = bool;
    type O = bool;
}