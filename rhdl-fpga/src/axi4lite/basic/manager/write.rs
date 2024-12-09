use crate::axi4lite::channel::receiver;
use crate::axi4lite::channel::sender;
use crate::axi4lite::types::WriteMISO;
use crate::axi4lite::types::WriteMOSI;
use crate::core::dff;
use rhdl::prelude::*;

use crate::axi4lite::types::ResponseKind;

// A basic manager...
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    // We need a sender for the address information
    addr: sender::U<Bits<32>>,
    // We need a sender for the data information
    data: sender::U<Bits<32>>,
    // We need a receiver for the response
    resp: receiver::U<ResponseKind>,
    // Overflow flag
    overflow: dff::U<bool>,
}

#[derive(Debug, Digital)]
pub struct I {
    pub axi: WriteMISO,
    pub cmd: Option<(b32, b32)>,
}

#[derive(Debug, Digital)]
pub struct O {
    pub axi: WriteMOSI,
    pub full: bool,
}

impl SynchronousIO for U {
    type I = I;
    type O = O;
    type Kernel = basic_write_manager_kernel;
}

#[kernel]
pub fn basic_write_manager_kernel(_cr: ClockReset, i: I, q: Q) -> (O, D) {
    let mut d = D::dont_care();
    let mut o = O::dont_care();
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
