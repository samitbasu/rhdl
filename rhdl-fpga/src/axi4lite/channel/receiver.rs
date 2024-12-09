use rhdl::prelude::*;

use crate::{core::option::pack, lid::option_carloni};

use super::{DataValid, Ready};

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<T: Digital + Default> {
    inner: option_carloni::U<T>,
}

#[derive(Debug, Digital)]
pub struct I<T: Digital> {
    // Connection to the bus
    pub bus: DataValid<T>,
    // Signal to allow more data to arrive
    pub ready: bool,
}

#[derive(Debug, Digital)]
pub struct O<T: Digital> {
    // Data from the bus - None if there is no data
    pub data: Option<T>,
    // Output connection to the bus
    pub bus: Ready,
}

impl<T: Digital + Default> SynchronousIO for U<T> {
    type I = I<T>;
    type O = O<T>;
    type Kernel = receiver_kernel<T>;
}

#[kernel]
pub fn receiver_kernel<T: Digital + Default>(cr: ClockReset, i: I<T>, q: Q<T>) -> (O<T>, D<T>) {
    let mut d = D::<T>::dont_care();
    let mut o = O::<T>::dont_care();
    d.inner.ready = i.ready;
    d.inner.data = pack::<T>(i.bus.valid, i.bus.data);
    o.data = q.inner.data;
    o.bus.ready = q.inner.ready;
    if cr.reset.any() {
        o.data = None;
    }
    (o, d)
}
