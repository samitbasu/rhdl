use std::ops::Add;

use crate::axi4lite::channel::receiver;
use crate::axi4lite::channel::sender;
use crate::core::dff;

use rhdl::prelude::*;

use crate::axi4lite::types::ReadDownstream;
use crate::axi4lite::types::ReadUpstream;
use crate::axi4lite::types::{Address, ReadResponse};

pub type ID = Bits<3>;
pub const ADDR: usize = 16;
pub type DATA = Bits<32>;

// A basic read manager
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    // We need a sender for the address information
    addr: sender::U<Address<ID, ADDR>>,
    // we need a receiver for the response
    data: receiver::U<ReadResponse<ID, DATA>>,
    // Address generator
    counter: dff::U<Bits<ADDR>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct I {
    pub axi: ReadUpstream<ID, DATA, ADDR>,
    pub run: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct O {
    pub axi: ReadDownstream<ID, ADDR>,
    pub data: Option<DATA>,
}

impl SynchronousIO for U {
    type I = I;
    type O = O;
    type Kernel = basic_read_manager_kernel;
}

type RA = Address<ID, ADDR>;

#[kernel]
pub fn basic_read_manager_kernel(cr: ClockReset, i: I, q: Q) -> (O, D) {
    let mut d = D::dont_care();
    let mut o = O::dont_care();
    d.addr.bus = i.axi.addr;
    d.data.bus = i.axi.data;
    d.addr.to_send = None;
    d.data.ready = true;
    if !q.addr.full && i.run {
        d.addr.to_send = Some(RA {
            id: bits(0),
            addr: q.counter,
        });
        d.counter = q.counter + 8;
    }
    o.axi.addr = q.addr.bus;
    o.axi.data = q.data.bus;
    o.data = match q.data.data {
        Some(response) => Some(response.data),
        None => None,
    };
    (o, d)
}
