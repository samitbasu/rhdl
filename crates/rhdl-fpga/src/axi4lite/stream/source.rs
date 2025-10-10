use rhdl::prelude::*;

use crate::axi4lite::channel::sender;

use super::{StreamMISO, StreamMOSI};

/// A source for an AXI stream.  In this case, it presents
/// a simple FIFO interface at the input, and then manages
/// the channel to send the data downstream.  If the downstream
/// component is not ready, eventually the `full` signal will
/// be set.  At this point, you cannot continue to send data.
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<T: Digital> {
    // We just need a sender for the data.  That's all there is to it.
    pub data: sender::U<T>,
}

#[derive(PartialEq, Debug, Digital, Clone)]
pub struct I<T: Digital> {
    /// The ready signal from the downstream component
    pub axi: StreamMISO,
    /// The data to be sent
    pub data: Option<T>,
}

#[derive(PartialEq, Debug, Digital, Clone)]
pub struct O<T: Digital> {
    /// The data signal to the downstream component
    pub axi: StreamMOSI<T>,
    /// The write-pipeline is full.  Do not write
    pub full: bool,
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
    d.data.bus.ready = i.axi.tready;
    o.axi.tvalid = q.data.bus.valid;
    o.axi.tdata = q.data.bus.data;
    // Feed the data into the sender
    d.data.to_send = i.data;
    o.full = q.data.full;
    (o, d)
}
