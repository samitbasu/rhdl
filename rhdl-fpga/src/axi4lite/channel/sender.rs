use rhdl::prelude::*;

use crate::{core::option::unpack, lid::option_carloni};

use super::{ChannelRToS, ChannelSToR};

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<T: Digital + Default> {
    inner: option_carloni::U<T>,
}

#[derive(Debug, Digital)]
pub struct I<T: Digital> {
    pub bus: ChannelRToS,
    pub to_send: Option<T>,
}

#[derive(Debug, Digital)]
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
    o.bus.data = T::default();
    o.bus.valid = false;
    if let Some(data) = q.inner.data {
        o.bus.data = data;
        o.bus.valid = true;
    }
    o.full = !q.inner.ready;
    (o, d)
}
