use crate::axi4lite::channel::receiver;
use crate::axi4lite::channel::sender;
use crate::axi4lite::types::result_to_read_response;
use crate::axi4lite::types::AXI4Error;
use crate::axi4lite::types::AxilAddr;
use crate::axi4lite::types::AxilData;
use crate::axi4lite::types::ReadMISO;
use crate::axi4lite::types::ReadMOSI;
use crate::axi4lite::types::ReadResponse;
use rhdl::prelude::*;

// Bridge for reads to a single cycle interface.

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    // We need a receiver for the address information
    cmd: receiver::U<AxilAddr>,
    // We need a sender for the response
    reply: sender::U<ReadResponse>,
}

#[derive(Debug, Digital)]
pub struct I {
    // AXI bus side of the bridge
    pub axi: ReadMOSI,
    // Provide a reply on this input for one cycle
    // to send a response.  Illegal if reply_full is true.
    pub reply: Option<Result<AxilData, AXI4Error>>,
    // Pulse this to accept the current cmd.
    // Illegal if cmd is None.
    pub cmd_next: bool,
}

#[derive(Debug, Digital)]
pub struct O {
    // AXI bus side of the bridge
    pub axi: ReadMISO,
    // The current command to be sent to the client
    // Held until acked by the `cmd_next` signal.
    pub cmd: Option<AxilAddr>,
    // If true, you cannot send a reply
    pub reply_full: bool,
}

impl SynchronousIO for U {
    type I = I;
    type O = O;
    type Kernel = read_bridge_kernel;
}

#[kernel]
pub fn read_bridge_kernel(cr: ClockReset, i: I, q: Q) -> (O, D) {
    let mut d = D::dont_care();
    let mut o = O::dont_care();
    // Connect the command bus
    d.cmd.bus.data = i.axi.araddr;
    d.cmd.bus.valid = i.axi.arvalid;
    o.axi.arready = q.cmd.bus.ready;
    // Connect the reply bus
    d.reply.bus.ready = i.axi.rready;
    o.axi.rdata = q.reply.bus.data.data;
    o.axi.rresp = q.reply.bus.data.resp;
    o.axi.rvalid = q.reply.bus.valid;
    // Feed the requested command out to the client
    o.cmd = q.cmd.data;
    // Tell the client if the reply sender is full
    o.reply_full = q.reply.full;
    // By default, we do not want to send data
    d.reply.to_send = None;
    // It is the clients responsibility to ensure that i.reply is None if
    // we indicate that reply is full.
    if let Some(resp) = i.reply {
        let axi_response = result_to_read_response(resp);
        d.reply.to_send = Some(axi_response);
    }
    // Feed the next command signal to the client
    d.cmd.next = i.cmd_next;
    if cr.reset.any() {
        o.cmd = None;
    }
    (o, d)
}
