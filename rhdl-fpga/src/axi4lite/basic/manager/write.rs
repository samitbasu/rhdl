use crate::axi4lite::channel::receiver;
use crate::axi4lite::channel::sender;
use crate::axi4lite::types::write_response_to_result;
use crate::axi4lite::types::AXI4Error;
use crate::axi4lite::types::AxilAddr;
use crate::axi4lite::types::StrobedData;
use crate::axi4lite::types::WriteCommand;
use crate::axi4lite::types::WriteMISO;
use crate::axi4lite::types::WriteMOSI;
use rhdl::prelude::*;

use crate::axi4lite::types::ResponseKind;

// A basic manager...
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    // We need a sender for the address information
    addr: sender::U<AxilAddr>,
    // We need a sender for the data information
    strobed_data: sender::U<StrobedData>,
    // We need a receiver for the response
    resp: receiver::U<ResponseKind>,
}

#[derive(PartialEq, Debug, Digital)]
pub struct I {
    // Bus side of the write manager
    pub axi: WriteMISO,
    // Provide a write command on this input for one cycle
    // if we are not full
    pub cmd: Option<WriteCommand>,
    // Accept the current reply on this cycle - valid
    // only if the reply is Some
    pub next: bool,
}

#[derive(PartialEq, Debug, Digital)]
pub struct O {
    // Bus side of the write manager
    pub axi: WriteMOSI,
    // The current write response provided by the client
    pub resp: Option<Result<(), AXI4Error>>,
    // If true, you cannot send a new write command to this manager
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
    d.strobed_data.bus.ready = i.axi.wready;
    o.axi.wdata = q.strobed_data.bus.data.data;
    o.axi.wstrb = q.strobed_data.bus.data.strobe;
    o.axi.wvalid = q.strobed_data.bus.valid;
    // Wire up the response bus
    d.resp.bus.data = i.axi.bresp;
    d.resp.bus.valid = i.axi.bvalid;
    o.axi.bready = q.resp.bus.ready;
    // Connect the command input to the address input
    // We can only accept new write commands if both
    // the data and address senders are not full
    o.full = q.addr.full || q.strobed_data.full;
    d.addr.to_send = None;
    d.strobed_data.to_send = None;
    // Requires client to pay attention to the FULL signal
    if let Some(write_cmd) = i.cmd {
        d.addr.to_send = Some(write_cmd.addr);
        d.strobed_data.to_send = Some(write_cmd.strobed_data);
    }
    o.resp = None;
    if let Some(response) = q.resp.data {
        o.resp = Some(write_response_to_result(response));
    }
    // Allow the client to acknowledge the response
    d.resp.next = i.next;
    (o, d)
}
