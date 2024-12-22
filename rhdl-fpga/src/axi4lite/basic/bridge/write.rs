use crate::axi4lite::channel::receiver;
use crate::axi4lite::channel::sender;
use crate::axi4lite::types::result_to_write_response;
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
    const DATA: usize,
    // AXI address width
    const ADDR: usize,
> {
    // We need a receiver for the address information
    addr: receiver::U<Bits<ADDR>>,
    // We need a receiver for the data information
    data: receiver::U<Bits<DATA>>,
    // We need a sender for the response
    resp: sender::U<ResponseKind>,
}

#[derive(Debug, Digital)]
pub struct D<const DATA: usize, const ADDR: usize> {
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
    // AXI bus side of the write bridge
    pub axi: WriteMOSI<DATA, ADDR>,
    // Provide a reply on this input for one cycle
    // to send a response.  Illegal if reply_full is true.
    pub reply: Option<Result<(), AXI4Error>>,
    // Pulse this to accept the current cmd.
    // Illegal if cmd is None.
    pub cmd_next: bool,
}

#[derive(Debug, Digital)]
pub struct O<const DATA: usize, const ADDR: usize> {
    // AXI bus side of the write bridge
    pub axi: WriteMISO,
    // The current command to be sent to the client
    // Held until acked by the `cmd_next` signal.
    pub cmd: Option<(Bits<ADDR>, Bits<DATA>)>,
    // If true, you cannot send a reply
    pub reply_full: bool,
}

impl<const DATA: usize, const ADDR: usize> SynchronousIO for U<DATA, ADDR> {
    type I = I<DATA, ADDR>;
    type O = O<DATA, ADDR>;
    type Kernel = write_bridge_kernel<DATA, ADDR>;
}

#[kernel]
pub fn write_bridge_kernel<const DATA: usize, const ADDR: usize>(
    _cr: ClockReset,
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
    o.cmd = None;
    let (addr_is_valid, addr) = unpack::<Bits<ADDR>>(q.addr.data);
    let (data_is_valid, data) = unpack::<Bits<DATA>>(q.data.data);
    // If the address is valid, and the data is valid, and the reply Q is not full,
    // then we can issue a write
    if addr_is_valid && data_is_valid {
        o.cmd = Some((addr, data));
    }
    // Let the client accept the command via the cmd_next signal
    d.addr.next = i.cmd_next;
    d.data.next = i.cmd_next;
    // If the client has a response to send, send it
    o.reply_full = q.resp.full;
    d.resp.to_send = if let Some(response) = i.reply {
        Some(result_to_write_response(response))
    } else {
        None
    };
    (o, d)
}
