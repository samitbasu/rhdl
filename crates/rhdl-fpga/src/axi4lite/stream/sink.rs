use rhdl::prelude::*;

use crate::axi4lite::channel::receiver;

use super::{StreamMISO, StreamMOSI};

/// A sink for an AXI stream.  In this case, it presents
/// a simple FIFO interface at the output, and then manages
/// the channel to receive the data from upstream.  If the
/// output component does not advance the stream sufficiently
/// quickly, eventually the pipeline will stall.
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<T: Digital> {
    // We just need a receiver for the data.  That's all there is to it.
    pub data: receiver::U<T>,
}

#[derive(PartialEq, Debug, Digital, Clone)]
pub struct I<T: Digital> {
    /// The data signals from the upstream component
    pub axi: StreamMOSI<T>,
    /// The advance signal to accept an element from the pipeline
    pub next: bool,
}

#[derive(PartialEq, Debug, Digital, Clone)]
pub struct O<T: Digital> {
    /// The ready signal to the upstream component
    pub axi: StreamMISO,
    /// The data from the pipeline
    pub data: Option<T>,
}

impl<T: Digital> SynchronousIO for U<T> {
    type I = I<T>;
    type O = O<T>;
    type Kernel = kernel<T>;
}

#[kernel]
pub fn kernel<T: Digital>(_cr: ClockReset, i: I<T>, q: Q<T>) -> (O<T>, D<T>) {
    let mut d = D::<T>::dont_care();
    let mut o = O::<T>::dont_care();
    // Wire up the data channel
    d.data.bus.valid = i.axi.tvalid;
    d.data.bus.data = i.axi.tdata;
    o.axi.tready = q.data.bus.ready;
    // Feed the data into the receiver
    d.data.next = i.next;
    o.data = q.data.data;
    (o, d)
}
