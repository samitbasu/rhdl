use crate::axi4lite::channel::receiver;
use crate::axi4lite::channel::sender;
use crate::axi4lite::types::write_response_to_result;
use crate::axi4lite::types::AXI4Error;
use crate::axi4lite::types::WriteMISO;
use crate::axi4lite::types::WriteMOSI;
use rhdl::prelude::*;

use crate::axi4lite::types::ResponseKind;

// A basic manager...
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<const DATA: usize, const ADDR: usize> {
    // We need a sender for the address information
    addr: sender::U<Bits<ADDR>>,
    // We need a sender for the data information
    data: sender::U<Bits<DATA>>,
    // We need a receiver for the response
    resp: receiver::U<ResponseKind>,
}

#[derive(Debug, Digital)]
pub struct I<const DATA: usize, const ADDR: usize> {
    // Bus side of the read manager
    pub axi: WriteMISO,
    // Provide a write command on this input for one cycle
    // if we are not full
    pub cmd: Option<(Bits<ADDR>, Bits<DATA>)>,
    // Accept the current reply on this cycle - valid
    // only if the reply is Some
    pub next: bool,
}

#[derive(Debug, Digital)]
pub struct O<const DATA: usize, const ADDR: usize> {
    // Bus side of the write manager
    pub axi: WriteMOSI<DATA, ADDR>,
    // The current write response provided by the client
    pub resp: Option<Result<(), AXI4Error>>,
    // If true, you cannot send a new write command to this manager
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
    // Connect the command input to the address input
    // We can only accept new write commands if both
    // the data and address senders are not full
    o.full = q.addr.full || q.data.full;
    d.addr.to_send = None;
    d.data.to_send = None;
    // Requires client to pay attention to the FULL signal
    if let Some((addr, data)) = i.cmd {
        d.addr.to_send = Some(addr);
        d.data.to_send = Some(data);
    }
    o.resp = None;
    if let Some(response) = q.resp.data {
        o.resp = Some(write_response_to_result(response));
    }
    // Allow the client to acknowledge the response
    d.resp.next = i.next;
    (o, d)
}
