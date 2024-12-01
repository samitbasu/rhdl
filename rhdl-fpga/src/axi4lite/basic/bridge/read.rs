use crate::axi4lite::channel::receiver;
use crate::axi4lite::channel::sender;
use crate::axi4lite::types::ResponseKind;
use crate::core::dff;
use crate::core::option::unpack;
use rhdl::prelude::*;

use crate::axi4lite::types::ReadDownstream;
use crate::axi4lite::types::ReadUpstream;
use crate::axi4lite::types::{Address, ReadResponse};

// Bridge for reads to a single cycle interface.

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<
    ID: Digital + Default, // The transaction ID type
    DATA: Digital,         // The data type stored in the memory
    const ADDR: usize = 32,
> {
    // We need a receiver for the address information
    addr: receiver::U<Address<ID, ADDR>>,
    // We need a sender for the response
    data: sender::U<ReadResponse<ID, DATA>>,
    // The pending transaction ID
    id: dff::U<Option<ID>>,
}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct I<ID: Digital, DATA: Digital, const ADDR: usize> {
    pub axi: ReadDownstream<ID, ADDR>,
    pub data: DATA,
}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct O<ID: Digital, DATA: Digital, const ADDR: usize> {
    pub axi: ReadUpstream<ID, DATA, ADDR>,
    pub read: Option<Bits<ADDR>>,
}

impl<ID: Digital + Default, DATA: Digital, const ADDR: usize> SynchronousIO for U<ID, DATA, ADDR> {
    type I = I<ID, DATA, ADDR>;
    type O = O<ID, DATA, ADDR>;
    type Kernel = read_bridge_kernel<ID, DATA, ADDR>;
}

#[kernel]
pub fn read_bridge_kernel<ID: Digital + Default, DATA: Digital, const ADDR: usize>(
    cr: ClockReset,
    i: I<ID, DATA, ADDR>,
    q: Q<ID, DATA, ADDR>,
) -> (O<ID, DATA, ADDR>, D<ID, DATA, ADDR>) {
    let mut d = D::<ID, DATA, ADDR>::dont_care();
    let mut o = O::<ID, DATA, ADDR>::dont_care();
    d.addr.bus = i.axi.addr;
    d.data.bus = i.axi.data;
    o.axi.addr = q.addr.bus;
    o.axi.data = q.data.bus;
    o.read = None;
    // By default, we halt the read operation when we have a new request.
    // This is because for the read to proceed, there must be an ability to send
    // the result.
    let (addr_is_valid, addr) = unpack::<Address<ID, ADDR>>(q.addr.data);
    d.addr.ready = !addr_is_valid;
    d.id = None;
    let (transaction_is_pending, tid) = unpack::<ID>(q.id);
    d.data.to_send = None;
    if transaction_is_pending && !q.data.full {
        d.data.to_send = Some(ReadResponse::<ID, DATA> {
            id: tid,
            data: i.data,
            resp: ResponseKind::OKAY,
        });
        d.id = None;
    }
    if addr_is_valid && !q.data.full {
        o.read = Some(addr.addr);
        d.addr.ready = true;
        d.id = Some(addr.id);
    }
    if cr.reset.any() {
        o.read = None;
    }
    (o, d)
}
