use crate::axi4lite::channel::receiver;
use crate::axi4lite::channel::sender;
use crate::axi4lite::types::ReadMISO;
use crate::axi4lite::types::ReadMOSI;
use crate::core::dff;

use rhdl::prelude::*;

use crate::axi4lite::types::ReadResponse;

// A basic read manager
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<const DATA: usize = 32, const ADDR: usize = 32> {
    // We need a sender for the address information
    addr: sender::U<Bits<ADDR>>,
    // we need a receiver for the response
    data: receiver::U<ReadResponse<DATA>>,
    // Overflow flag
    overflow: dff::U<bool>,
}

#[derive(Debug, Digital)]
pub struct I<const DATA: usize, const ADDR: usize> {
    pub axi: ReadMISO<DATA>,
    pub cmd: Option<Bits<ADDR>>,
}

#[derive(Debug, Digital)]
pub struct O<const DATA: usize, const ADDR: usize> {
    pub axi: ReadMOSI<ADDR>,
    pub data: Option<Bits<DATA>>,
    pub full: bool,
}

impl<const DATA: usize, const ADDR: usize> SynchronousIO for U<DATA, ADDR> {
    type I = I<DATA, ADDR>;
    type O = O<DATA, ADDR>;
    type Kernel = basic_read_manager_kernel<DATA, ADDR>;
}

#[kernel]
#[allow(clippy::manual_map)]
pub fn basic_read_manager_kernel<const DATA: usize, const ADDR: usize>(
    _cr: ClockReset,
    i: I<DATA, ADDR>,
    q: Q<DATA, ADDR>,
) -> (O<DATA, ADDR>, D<DATA, ADDR>) {
    let mut d = D::<DATA, ADDR>::dont_care();
    let mut o = O::<DATA, ADDR>::dont_care();
    // Wire up the address bus
    d.addr.bus.ready = i.axi.arready;
    o.axi.araddr = q.addr.bus.data;
    o.axi.arvalid = q.addr.bus.valid;
    // Wire up the data response bus
    d.data.bus.data.data = i.axi.rdata;
    d.data.bus.data.resp = i.axi.rresp;
    d.data.bus.valid = i.axi.rvalid;
    o.axi.rready = q.data.bus.ready;
    d.addr.to_send = None;
    d.data.ready = true;
    d.overflow = q.overflow;
    let is_full = q.overflow || q.addr.full;
    if let Some(addr) = i.cmd {
        if !is_full {
            d.addr.to_send = Some(addr);
        } else {
            d.overflow = true;
        }
    }
    o.data = match q.data.data {
        Some(response) => Some(response.data),
        None => None,
    };
    o.full = is_full;
    (o, d)
}
