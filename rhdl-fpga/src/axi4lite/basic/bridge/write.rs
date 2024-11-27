use crate::axi4lite::channel::receiver;
use crate::axi4lite::channel::sender;
use crate::axi4lite::types::ResponseKind;
use crate::core::option::unpack;
use rhdl::prelude::*;

use crate::axi4lite::types::WriteDownstream;
use crate::axi4lite::types::WriteUpstream;
use crate::axi4lite::types::{Address, WriteResponse};

// Bridge for writes to a single cycle interface.

#[derive(Clone, Debug, Synchronous, Default)]
pub struct U<
    // Transaction ID type
    ID: Digital,
    // Data type stored in the memory
    DATA: Digital,
    // AXI address width
    const ADDR: usize = 32,
> {
    // We need a receiver for the address information
    addr: receiver::U<Address<ID, ADDR>>,
    // We need a receiver for the data information
    data: receiver::U<DATA>,
    // We need a sender for the response
    resp: sender::U<WriteResponse<ID>>,
}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct D<ID: Digital, DATA: Digital, const ADDR: usize> {
    pub addr: receiver::I<Address<ID, ADDR>>,
    pub data: receiver::I<DATA>,
    pub resp: sender::I<WriteResponse<ID>>,
}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct Q<ID: Digital, DATA: Digital, const ADDR: usize> {
    pub addr: receiver::O<Address<ID, ADDR>>,
    pub data: receiver::O<DATA>,
    pub resp: sender::O<WriteResponse<ID>>,
}

impl<ID: Digital, DATA: Digital, const ADDR: usize> SynchronousDQ for U<ID, DATA, ADDR> {
    type D = D<ID, DATA, ADDR>;
    type Q = Q<ID, DATA, ADDR>;
}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct I<ID: Digital, DATA: Digital, const ADDR: usize> {
    pub axi: WriteDownstream<ID, DATA, ADDR>,
    pub full: bool,
}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct O<ID: Digital, DATA: Digital, const ADDR: usize> {
    pub axi: WriteUpstream<ID, ADDR>,
    pub write: Option<(Bits<ADDR>, DATA)>,
}

impl<ID: Digital, DATA: Digital, const ADDR: usize> SynchronousIO for U<ID, DATA, ADDR> {
    type I = I<ID, DATA, ADDR>;
    type O = O<ID, DATA, ADDR>;
    type Kernel = write_bridge_kernel<ID, DATA, ADDR>;
}

#[kernel]
pub fn write_bridge_kernel<ID: Digital, DATA: Digital, const ADDR: usize>(
    cr: ClockReset,
    i: I<ID, DATA, ADDR>,
    q: Q<ID, DATA, ADDR>,
) -> (O<ID, DATA, ADDR>, D<ID, DATA, ADDR>) {
    let mut d = D::<ID, DATA, ADDR>::init();
    let mut o = O::<ID, DATA, ADDR>::init();
    d.addr.bus = i.axi.addr;
    d.data.bus = i.axi.data;
    d.resp.bus = i.axi.resp;
    d.resp.to_send = None;
    o.axi.addr = q.addr.bus;
    o.axi.data = q.data.bus;
    o.axi.resp = q.resp.bus;
    o.write = None;
    // Connect the ready signal so that we stop when
    // an address arrives.
    let (addr_is_valid, addr) = unpack::<Address<ID, ADDR>>(q.addr.data);
    d.addr.ready = !addr_is_valid;
    // Same for the data
    let (data_is_valid, data) = unpack::<DATA>(q.data.data);
    d.data.ready = !data_is_valid;
    // If both address and data are valid and the response channel is free, issue a write
    if addr_is_valid && data_is_valid && !q.resp.full && !i.full {
        o.write = Some((addr.addr, data));
        // We do not need to hold them any longer
        d.addr.ready = true;
        d.data.ready = true;
        d.resp.to_send = Some(WriteResponse::<ID> {
            id: addr.id,
            resp: ResponseKind::OKAY,
        })
    }
    if cr.reset.any() {
        o.write = None;
    }
    (o, d)
}
