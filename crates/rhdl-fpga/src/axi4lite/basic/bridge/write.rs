use crate::axi4lite::channel::receiver;
use crate::axi4lite::channel::sender;
use crate::axi4lite::types::result_to_write_response;
use crate::axi4lite::types::AXI4Error;
use crate::axi4lite::types::AxilAddr;
use crate::axi4lite::types::ResponseKind;
use crate::axi4lite::types::StrobedData;
use crate::axi4lite::types::WriteCommand;
use crate::axi4lite::types::WriteMISO;
use crate::axi4lite::types::WriteMOSI;
use crate::core::option::unpack;
use rhdl::prelude::*;

// Bridge for writes to a single cycle interface.

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    // We need a receiver for the address information
    addr: receiver::U<AxilAddr>,
    // We need a receiver for the data information (includes the strobe)
    strobed_data: receiver::U<StrobedData>,
    // We need a sender for the response
    resp: sender::U<ResponseKind>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct I {
    // AXI bus side of the write bridge
    pub axi: WriteMOSI,
    // Provide a reply on this input for one cycle
    // to send a response.  Illegal if reply_full is true.
    pub reply: Option<Result<(), AXI4Error>>,
    // Pulse this to accept the current cmd.
    // Illegal if cmd is None.
    pub cmd_next: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct O {
    // AXI bus side of the write bridge
    pub axi: WriteMISO,
    // The current command to be sent to the client
    // Held until acked by the `cmd_next` signal.
    pub cmd: Option<WriteCommand>,
    // If true, you cannot send a reply
    pub reply_full: bool,
}

impl SynchronousIO for U {
    type I = I;
    type O = O;
    type Kernel = write_bridge_kernel;
}

#[kernel]
pub fn write_bridge_kernel(_cr: ClockReset, i: I, q: Q) -> (O, D) {
    let mut d = D::dont_care();
    let mut o = O::dont_care();
    // Connect the address channel
    d.addr.bus.data = i.axi.awaddr;
    d.addr.bus.valid = i.axi.awvalid;
    o.axi.awready = q.addr.bus.ready;
    // Connect the data channel
    d.strobed_data.bus.data.data = i.axi.wdata;
    d.strobed_data.bus.data.strobe = i.axi.wstrb;
    d.strobed_data.bus.valid = i.axi.wvalid;
    o.axi.wready = q.strobed_data.bus.ready;
    // Connect the response channel
    d.resp.bus.ready = i.axi.bready;
    o.axi.bresp = q.resp.bus.data;
    o.axi.bvalid = q.resp.bus.valid;
    o.cmd = None;
    let (addr_is_valid, addr) = unpack::<AxilAddr>(q.addr.data);
    let (data_is_valid, strobed_data) = unpack::<StrobedData>(q.strobed_data.data);
    // If the address is valid, and the data is valid, and the reply Q is not full,
    // then we can issue a write
    if addr_is_valid && data_is_valid {
        o.cmd = Some(WriteCommand { addr, strobed_data });
    }
    // Let the client accept the command via the cmd_next signal
    d.addr.next = i.cmd_next;
    d.strobed_data.next = i.cmd_next;
    // If the client has a response to send, send it
    o.reply_full = q.resp.full;
    d.resp.to_send = if let Some(response) = i.reply {
        Some(result_to_write_response(response))
    } else {
        None
    };
    (o, d)
}
