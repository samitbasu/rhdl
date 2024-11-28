use crate::axi4lite::channel::receiver;
use crate::axi4lite::channel::sender;
use crate::core::dff;
use rhdl::prelude::*;

use crate::axi4lite::types::WriteDownstream;
use crate::axi4lite::types::WriteUpstream;
use crate::axi4lite::types::{Address, BurstData, WriteResponse};

pub type ID = Bits<3>;
pub const ADDR: usize = 8;
pub type DATA = Bits<32>;

// A basic manager...
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    // We need a sender for the address information
    addr: sender::U<Address<ID, ADDR>>,
    // We need a sender for the data information
    data: sender::U<DATA>,
    // We need a receiver for the response
    resp: receiver::U<WriteResponse<ID>>,
    // Counter to hold the data
    counter: dff::U<DATA>,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct I {
    pub axi: WriteUpstream<ID, ADDR>,
    pub run: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct O {
    pub axi: WriteDownstream<ID, DATA, ADDR>,
}

impl SynchronousIO for U {
    type I = I;
    type O = O;
    type Kernel = basic_write_manager_kernel;
}

type WA = Address<ID, ADDR>;
type BD = BurstData<DATA>;

#[kernel]
pub fn basic_write_manager_kernel(cr: ClockReset, i: I, q: Q) -> (O, D) {
    let mut d = D::maybe_init();
    let mut o = O::maybe_init();
    d.addr.bus = i.axi.addr;
    d.data.bus = i.axi.data;
    d.resp.bus = i.axi.resp;
    d.addr.to_send = None;
    d.data.to_send = None;
    d.resp.ready = true;
    d.counter = q.counter;
    if !q.addr.full && !q.data.full && i.run {
        d.addr.to_send = Some(WA {
            id: bits(0),
            addr: bits(42),
        });
        d.data.to_send = Some(q.counter);
        d.counter = q.counter + 1;
    }
    o.axi.addr = q.addr.bus;
    o.axi.data = q.data.bus;
    o.axi.resp = q.resp.bus;
    (o, d)
}
