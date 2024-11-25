use rhdl::prelude::*;

use crate::{core::option::unpack, lid::option_carloni};

use super::{ChannelMToS, ChannelSToM};

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<T: Digital> {
    inner: option_carloni::U<T>,
}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct I<T: Digital> {
    pub bus: ChannelSToM,
    pub to_send: Option<T>,
}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct O<T: Digital> {
    pub bus: ChannelMToS<T>,
    pub full: bool,
}

impl<T: Digital> SynchronousIO for U<T> {
    type I = I<T>;
    type O = O<T>;
    type Kernel = sender_kernel<T>;
}

//    i           inner           bus
//  to_send -----> data ------> data/valid
//        q        d   q        o
//
//    full <----- !ready <----- ready
//

#[kernel]
pub fn sender_kernel<T: Digital>(cr: ClockReset, i: I<T>, q: Q<T>) -> (O<T>, D<T>) {
    let mut d = D::<T>::init();
    let mut o = O::<T>::init();
    // Forward the to_send to the inner module
    d.inner.data = i.to_send;
    d.inner.ready = i.bus.ready;
    let (is_valid, data) = unpack::<T>(q.inner.data);
    o.bus.data = data;
    o.bus.valid = is_valid;
    o.full = !q.inner.ready;
    (o, d)
}
