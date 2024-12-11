use crate::axi4lite::channel::receiver;
use crate::axi4lite::channel::sender;
use crate::axi4lite::types::WriteMISO;
use crate::axi4lite::types::WriteMOSI;
use crate::core::dff;
use rhdl::prelude::*;

use crate::axi4lite::types::ResponseKind;

// A basic manager...
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<const DATA: usize = 32, const ADDR: usize = 32> {
    // We need a sender for the address information
    addr: sender::U<Bits<ADDR>>,
    // We need a sender for the data information
    data: sender::U<Bits<DATA>>,
    // We need a receiver for the response
    resp: receiver::U<ResponseKind>,
    // Overflow flag
    overflow: dff::U<bool>,
}

#[derive(Debug, Digital)]
pub struct I<const DATA: usize, const ADDR: usize> {
    pub axi: WriteMISO,
    pub cmd: Option<(Bits<ADDR>, Bits<DATA>)>,
}

#[derive(Debug, Digital)]
pub struct O<const DATA: usize, const ADDR: usize> {
    pub axi: WriteMOSI<DATA, ADDR>,
    pub full: bool,
}

impl<const DATA: usize, const ADDR: usize> SynchronousIO for U<DATA, ADDR> {
    type I = I<DATA, ADDR>;
    type O = O<DATA, ADDR>;
    type Kernel = basic_write_manager_kernel<DATA, ADDR>;
}

#[kernel]
pub fn basic_write_manager_kernel<const DATA: usize, const ADDR: usize>(
    _cr: ClockReset,
    i: I<DATA, ADDR>,
    q: Q<DATA, ADDR>,
) -> (O<DATA, ADDR>, D<DATA, ADDR>) {
    let mut d = D::<DATA, ADDR>::dont_care();
    let mut o = O::<DATA, ADDR>::dont_care();
    // Wire up the address bus
    d.addr.bus.ready = i.axi.awready;
    o.axi.awaddr = q.addr.bus.data;
    o.axi.awvalid = q.addr.bus.valid;
    // Wire up the data bus
    d.data.bus.ready = i.axi.wready;
    o.axi.wdata = q.data.bus.data;
    o.axi.wvalid = q.data.bus.valid;
    // Wire up the response bus
    d.resp.bus.data = i.axi.bresp;
    d.resp.bus.valid = i.axi.bvalid;
    o.axi.bready = q.resp.bus.ready;
    d.addr.to_send = None;
    d.data.to_send = None;
    d.resp.ready = true;
    d.overflow = q.overflow;
    let is_full = q.overflow || q.addr.full | q.data.full;
    if let Some((addr, data)) = i.cmd {
        if !is_full {
            d.addr.to_send = Some(addr);
            d.data.to_send = Some(data);
        } else {
            d.overflow = true;
        }
    }
    o.full = is_full;
    (o, d)
}
