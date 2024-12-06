use rhdl::prelude::*;

use crate::core::option::{pack, unpack};

use super::carloni;

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<T: Digital + Default> {
    inner: carloni::U<T>,
}

#[derive(Debug, Digital)]
pub struct I<T: Digital> {
    pub data: Option<T>,
    pub ready: bool,
}

#[derive(Debug, Digital)]
pub struct O<T: Digital> {
    pub data: Option<T>,
    pub ready: bool,
}

impl<T: Digital + Default> SynchronousIO for U<T> {
    type I = I<T>;
    type O = O<T>;
    type Kernel = option_carloni_kernel<T>;
}

#[kernel]
pub fn option_carloni_kernel<T: Digital + Default>(
    _cr: ClockReset,
    i: I<T>,
    q: Q<T>,
) -> (O<T>, D<T>) {
    let mut d = D::<T>::dont_care();
    let (data_valid, data) = unpack::<T>(i.data);
    d.inner.data_in = data;
    d.inner.void_in = !data_valid;
    d.inner.stop_in = !i.ready;
    let mut o = O::<T>::dont_care();
    o.ready = !q.inner.stop_out;
    o.data = pack::<T>(!q.inner.void_out, q.inner.data_out);
    (o, d)
}
