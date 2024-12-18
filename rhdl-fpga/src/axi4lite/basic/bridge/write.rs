use crate::axi4lite::channel::receiver;
use crate::axi4lite::channel::sender;
use crate::axi4lite::types::response_codes;
use crate::axi4lite::types::AXI4Error;
use crate::axi4lite::types::ResponseKind;
use crate::axi4lite::types::WriteMISO;
use crate::axi4lite::types::WriteMOSI;
use crate::core::option::unpack;
use rhdl::prelude::*;

// Bridge for writes to a single cycle interface.

#[derive(Clone, Debug, Synchronous, Default)]
pub struct U<
    // AXI data width
    const DATA: usize = 32,
    // AXI address width
    const ADDR: usize = 32,
> {
    // We need a receiver for the address information
    addr: receiver::U<Bits<ADDR>>,
    // We need a receiver for the data information
    data: receiver::U<Bits<DATA>>,
    // We need a sender for the response
    resp: sender::U<ResponseKind>,
}

#[derive(Debug, Digital)]
pub struct D<const DATA: usize = 32, const ADDR: usize = 32> {
    pub addr: receiver::I<Bits<ADDR>>,
    pub data: receiver::I<Bits<DATA>>,
    pub resp: sender::I<ResponseKind>,
}

#[derive(Debug, Digital)]
pub struct Q<const DATA: usize, const ADDR: usize> {
    pub addr: receiver::O<Bits<ADDR>>,
    pub data: receiver::O<Bits<DATA>>,
    pub resp: sender::O<ResponseKind>,
}

impl<const DATA: usize, const ADDR: usize> SynchronousDQ for U<DATA, ADDR> {
    type D = D<DATA, ADDR>;
    type Q = Q<DATA, ADDR>;
}

#[derive(Debug, Digital)]
pub struct I<const DATA: usize, const ADDR: usize> {
    pub axi: WriteMOSI<DATA, ADDR>,
    pub response: Option<Result<(), AXI4Error>>,
    pub full: bool,
}

#[derive(Debug, Digital)]
pub struct O<const DATA: usize, const ADDR: usize> {
    pub axi: WriteMISO,
    pub write: Option<(Bits<ADDR>, Bits<DATA>)>,
}

impl<const DATA: usize, const ADDR: usize> SynchronousIO for U<DATA, ADDR> {
    type I = I<DATA, ADDR>;
    type O = O<DATA, ADDR>;
    type Kernel = write_bridge_kernel<DATA, ADDR>;
}

#[kernel]
pub fn write_bridge_kernel<const DATA: usize, const ADDR: usize>(
    cr: ClockReset,
    i: I<DATA, ADDR>,
    q: Q<DATA, ADDR>,
) -> (O<DATA, ADDR>, D<DATA, ADDR>) {
    let mut d = D::<DATA, ADDR>::dont_care();
    let mut o = O::<DATA, ADDR>::dont_care();
    // Connect the address channel
    d.addr.bus.data = i.axi.awaddr;
    d.addr.bus.valid = i.axi.awvalid;
    o.axi.awready = q.addr.bus.ready;
    // Connect the data channel
    d.data.bus.data = i.axi.wdata;
    d.data.bus.valid = i.axi.wvalid;
    o.axi.wready = q.data.bus.ready;
    // Connect the response channel
    d.resp.bus.ready = i.axi.bready;
    o.axi.bresp = q.resp.bus.data;
    o.axi.bvalid = q.resp.bus.valid;
    d.resp.to_send = None;
    o.write = None;
    // Connect the ready signal so that we stop when
    // an address arrives.
    let (addr_is_valid, addr) = unpack::<Bits<ADDR>>(q.addr.data);
    d.addr.ready = !addr_is_valid;
    // Same for the data
    let (data_is_valid, data) = unpack::<Bits<DATA>>(q.data.data);
    d.data.ready = !data_is_valid;
    // If both address and data are valid and the response channel is free, issue a write
    if addr_is_valid && data_is_valid && !i.full {
        o.write = Some((addr, data));
        // We do not need to hold them any longer
        d.addr.ready = true;
        d.data.ready = true;
    }
    // Forward the response to the sender
    d.resp.to_send = match i.response {
        Some(Ok::<(), AXI4Error>(())) => Some(response_codes::OKAY),
        Some(Err(e)) => match e {
            AXI4Error::SLVERR => Some(response_codes::SLVERR),
            AXI4Error::DECERR => Some(response_codes::DECERR),
        },
        None => None,
    };
    if cr.reset.any() {
        o.write = None;
    }
    (o, d)
}
