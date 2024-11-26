use std::io::Read;

use crate::axi4lite::channel::receiver;
use crate::axi4lite::channel::sender;
use crate::axi4lite::types::ResponseKind;
use crate::core::dff;
use crate::core::option::pack;
use crate::core::option::unpack;
use rhdl::prelude::*;

use crate::axi4lite::types::ReadDownstream;
use crate::axi4lite::types::ReadUpstream;
use crate::axi4lite::types::{Address, ReadResponse};

// Bridge for reads to a single cycle interface.

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<ID: Digital, DATA: Digital, const ADDR: usize> {
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

impl<ID: Digital, DATA: Digital, const ADDR: usize> SynchronousIO for U<ID, DATA, ADDR> {
    type I = I<ID, DATA, ADDR>;
    type O = O<ID, DATA, ADDR>;
    type Kernel = read_bridge_kernel<ID, DATA, ADDR>;
}

#[kernel]
pub fn read_bridge_kernel<ID: Digital, DATA: Digital, const ADDR: usize>(
    cr: ClockReset,
    i: I<ID, DATA, ADDR>,
    q: Q<ID, DATA, ADDR>,
) -> (O<ID, DATA, ADDR>, D<ID, DATA, ADDR>) {
    let mut d = D::<ID, DATA, ADDR>::init();
    let mut o = O::<ID, DATA, ADDR>::init();
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
    // In the write case, we could make all of the assertions at once.  That the
    // data was valid, that the address was valid, and that the
    // response channel was not full.  In the read case, we need to check
    // that the read channel will not be full in the next clock cycle.  Not
    // the current one.  While this _could_ be done with some fancy logic,
    // it is easier to just check that the response channel is not full now.
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
