use crate::axi4lite::channel::receiver;
use crate::axi4lite::channel::sender;
use crate::axi4lite::types::read_response_to_result;
use crate::axi4lite::types::AXI4Error;
use crate::axi4lite::types::AxilAddr;
use crate::axi4lite::types::AxilData;
use crate::axi4lite::types::ReadMISO;
use crate::axi4lite::types::ReadMOSI;

use rhdl::prelude::*;

use crate::axi4lite::types::ReadResponse;

// A basic read manager
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    // We need a sender for the address information
    addr: sender::U<AxilAddr>,
    // we need a receiver for the response
    data: receiver::U<ReadResponse>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct I {
    // Bus side of the manager
    pub axi: ReadMISO,
    // Provide a read command on this input for one cycle
    // if we are not full
    pub cmd: Option<AxilAddr>,
    // Accept the current reply on this cycle - valid
    // only if the reply is Some
    pub next: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct O {
    // Bus side of the manager
    pub axi: ReadMOSI,
    // The current data reply provided by the client
    pub data: Option<Result<AxilData, AXI4Error>>,
    // If true, you cannot send a new read command to this manager
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
    // Connect the command input to the address input
    d.addr.to_send = i.cmd;
    // Tell the client if the sender is full
    o.full = q.addr.full;
    // Connect the reply output to the receiver
    o.data = None;
    if let Some(response) = q.data.data {
        o.data = Some(read_response_to_result(response));
    }
    // Allow the client to acknowledge the response
    d.data.next = i.next;
    (o, d)
}
