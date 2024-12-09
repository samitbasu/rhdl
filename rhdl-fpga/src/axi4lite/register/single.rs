// A simple single register with an AXI bus interface for reading
// and writing it.  Ignores the address information (i.e., the
// register is read or written for any address in the address space
// of the bus).  This is fine, since it is the responsibility of the
// interconnect to ensure non-overlapping address spaces.
use crate::{
    axi4lite::{
        basic::bridge,
        types::{MISO, MOSI},
    },
    core::dff,
};
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<const REG_WIDTH: usize = 32, const DATA: usize = 32, const ADDR: usize = 32> {
    // We need a read bridge
    read_bridge: bridge::read::U<DATA, ADDR>,
    // And a register to hold the value
    reg: dff::U<Bits<REG_WIDTH>>,
    // And a write bridge
    write_bridge: bridge::write::U<DATA, ADDR>,
}

impl<const REG_WIDTH: usize, const DATA: usize, const ADDR: usize> SynchronousIO
    for U<REG_WIDTH, DATA, ADDR>
{
    type I = MOSI<DATA, ADDR>;
    type O = MISO<DATA>;
    type Kernel = single_kernel<REG_WIDTH, DATA, ADDR>;
}

#[kernel]
pub fn single_kernel<const REG_WIDTH: usize, const DATA: usize, const ADDR: usize>(
    _cr: ClockReset,
    i: MOSI<DATA, ADDR>,
    q: Q<REG_WIDTH, DATA, ADDR>,
) -> (MISO<DATA>, D<REG_WIDTH, DATA, ADDR>) {
    let mut d = D::<REG_WIDTH, DATA, ADDR>::dont_care();
    let mut o = MISO::<DATA>::dont_care();
    // Connect the read bridge inputs and outputs to the bus
    d.read_bridge.axi = i.read;
    o.read = q.read_bridge.axi;
    // Connect the write bridge inputs and outputs to the bus
    d.write_bridge.axi = i.write;
    o.write = q.write_bridge.axi;
    // Connect the register
    d.reg = q.reg;
    // Connect the read bridge's input to the register
    // The read bridge's address is ignored
    d.read_bridge.data = q.reg.resize();
    // Connect the write bridge's output to the register
    if let Some((_addr, value)) = q.write_bridge.write {
        d.reg = value.resize();
    }
    (o, d)
}
