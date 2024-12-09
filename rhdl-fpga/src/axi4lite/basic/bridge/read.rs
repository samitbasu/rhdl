use crate::axi4lite::channel::receiver;
use crate::axi4lite::channel::sender;
use crate::axi4lite::types::response_codes;
use crate::axi4lite::types::ReadMISO;
use crate::axi4lite::types::ReadMOSI;
use crate::axi4lite::types::ReadResponse;
use crate::core::dff;
use crate::core::option::unpack;
use rhdl::prelude::*;

// Bridge for reads to a single cycle interface.

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<const DATA: usize = 32, const ADDR: usize = 32> {
    // We need a receiver for the address information
    addr: receiver::U<Bits<ADDR>>,
    // We need a sender for the response
    data: sender::U<ReadResponse<DATA>>,
    // The pending transaction flag
    id: dff::U<Option<()>>,
}

#[derive(Debug, Digital)]
pub struct I<const DATA: usize = 32, const ADDR: usize = 32> {
    pub axi: ReadMOSI<ADDR>,
    pub data: Bits<DATA>,
}

#[derive(Debug, Digital)]
pub struct O<const DATA: usize = 32, const ADDR: usize = 32> {
    pub axi: ReadMISO<DATA>,
    pub read: Option<Bits<ADDR>>,
}

impl<const DATA: usize, const ADDR: usize> SynchronousIO for U<DATA, ADDR> {
    type I = I<DATA, ADDR>;
    type O = O<DATA, ADDR>;
    type Kernel = read_bridge_kernel<DATA, ADDR>;
}

#[kernel]
pub fn read_bridge_kernel<const DATA: usize, const ADDR: usize>(
    cr: ClockReset,
    i: I<DATA, ADDR>,
    q: Q<DATA, ADDR>,
) -> (O<DATA, ADDR>, D<DATA, ADDR>) {
    let mut d = D::<DATA, ADDR>::dont_care();
    let mut o = O::<DATA, ADDR>::dont_care();
    d.addr.bus.data = i.axi.araddr;
    d.addr.bus.valid = i.axi.arvalid;
    o.axi.arready = q.addr.bus.ready;
    d.data.bus.ready = i.axi.rready;
    o.axi.rdata = q.data.bus.data.data;
    o.axi.rresp = q.data.bus.data.resp;
    o.axi.rvalid = q.data.bus.valid;
    o.read = None;
    // By default, we halt the read operation when we have a new request.
    // This is because for the read to proceed, there must be an ability to send
    // the result.
    let (addr_is_valid, addr) = unpack::<Bits<ADDR>>(q.addr.data);
    d.addr.ready = !addr_is_valid;
    d.id = None;
    let transaction_is_pending = match q.id {
        Some(_x) => true,
        None => false,
    };
    d.data.to_send = None;
    if transaction_is_pending && !q.data.full {
        d.data.to_send = Some(ReadResponse::<DATA> {
            data: i.data,
            resp: response_codes::OKAY,
        });
        d.id = None;
    }
    if addr_is_valid && !q.data.full {
        o.read = Some(addr);
        d.addr.ready = true;
        d.id = Some(());
    }
    if cr.reset.any() {
        o.read = None;
    }
    (o, d)
}
