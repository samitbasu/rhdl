use rhdl::prelude::*;

use crate::{core::option::unpack, lid::option_carloni};

use super::{ChannelRToS, ChannelSToR};

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<T: Digital + Default> {
    inner: option_carloni::U<T>,
}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct I<T: Digital> {
    pub bus: ChannelRToS,
    pub to_send: Option<T>,
}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct O<T: Digital> {
    pub bus: ChannelSToR<T>,
    pub full: bool,
}

impl<T: Digital + Default> SynchronousIO for U<T> {
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
pub fn sender_kernel<T: Digital + Default>(cr: ClockReset, i: I<T>, q: Q<T>) -> (O<T>, D<T>) {
    let mut d = D::<T>::dont_care();
    let mut o = O::<T>::dont_care();
    // Forward the to_send to the inner module
    d.inner.data = i.to_send;
    d.inner.ready = i.bus.ready;
    let (is_valid, data) = unpack::<T>(q.inner.data);
    o.bus.data = data;
    o.bus.valid = is_valid;
    o.full = !q.inner.ready;
    (o, d)
}
