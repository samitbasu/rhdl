use rhdl::prelude::*;

use crate::{core::option::pack, lid::option_carloni};

use super::{ChannelMToS, ChannelSToM};

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<T: Digital> {
    inner: option_carloni::U<T>,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct I<T: Digital> {
    // Connection to the bus
    pub bus: ChannelMToS<T>,
    // Signal to accept the data.
    pub next: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct O<T: Digital> {
    // Data from the bus - None if there is no data
    pub data: Option<T>,
    // Output connection to the bus
    pub bus: ChannelSToM,
}

impl<T: Digital> SynchronousIO for U<T> {
    type I = I<T>;
    type O = O<T>;
    type Kernel = receiver_kernel<T>;
}

#[kernel]
pub fn receiver_kernel<T: Digital>(_cr: ClockReset, i: I<T>, q: Q<T>) -> (O<T>, D<T>) {
    let mut d = D::<T>::init();
    let mut o = O::<T>::init();
    d.inner.ready = i.next;
    d.inner.data = pack::<T>(i.bus.valid, i.bus.data);
    o.data = q.inner.data;
    o.bus.ready = q.inner.ready;
    (o, d)
}
