use crate::axi4lite::channel::receiver;
use crate::axi4lite::channel::sender;
use crate::axi4lite::types::ReadMISO;
use crate::axi4lite::types::ReadMOSI;
use crate::core::dff;

use rhdl::prelude::*;

use crate::axi4lite::types::ReadResponse;

// A basic read manager
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    // We need a sender for the address information
    addr: sender::U<Bits<32>>,
    // we need a receiver for the response
    data: receiver::U<ReadResponse<32>>,
    // Overflow flag
    overflow: dff::U<bool>,
}

#[derive(Debug, Digital)]
pub struct I {
    pub axi: ReadMISO,
    pub cmd: Option<b32>,
}

#[derive(Debug, Digital)]
pub struct O {
    pub axi: ReadMOSI,
    pub data: Option<b32>,
    pub full: bool,
}

impl SynchronousIO for U {
    type I = I;
    type O = O;
    type Kernel = basic_read_manager_kernel;
}

#[kernel]
#[allow(clippy::manual_map)]
pub fn basic_read_manager_kernel(_cr: ClockReset, i: I, q: Q) -> (O, D) {
    let mut d = D::dont_care();
    let mut o = O::dont_care();
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
