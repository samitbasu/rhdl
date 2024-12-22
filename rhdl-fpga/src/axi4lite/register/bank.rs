use rhdl::prelude::*;

use crate::{
    axi4lite::{
        basic::bridge,
        types::{AXI4Error, MISO, MOSI},
    },
    core::{constant, dff, option::unpack},
};

// Each register is at a different word address

#[derive(Clone, Debug, SynchronousDQ, Synchronous)]
pub struct U<const BANK_SIZE: usize, const REG_WIDTH: usize, const DATA: usize, const ADDR: usize> {
    // We need a read bridge
    read_bridge: bridge::read::U<DATA, ADDR>,
    // And a set of registers to hold the values
    reg: [dff::U<Bits<REG_WIDTH>>; BANK_SIZE],
    // And a write bridge
    write_bridge: bridge::write::U<DATA, ADDR>,
    // The right shift amount to apply to the byte address
    // to get the reg address
    right_shift: constant::U<Bits<8>>,
}

fn find_addr_shift(data: usize) -> u128 {
    match data {
        8 => 0,
        16 => 1,
        32 => 2,
        64 => 3,
        _ => panic!("Unsupported data width"),
    }
}

impl<const BANK_SIZE: usize, const REG_WIDTH: usize, const DATA: usize, const ADDR: usize> Default
    for U<BANK_SIZE, REG_WIDTH, DATA, ADDR>
{
    fn default() -> Self {
        // The right shift should be the number of bits in DATA as bytes as a log2 shift
        let right_shift = find_addr_shift(DATA);
        Self {
            read_bridge: Default::default(),
            reg: array_init::array_init(|_| Default::default()),
            write_bridge: Default::default(),
            right_shift: constant::U::new(bits(right_shift)),
        }
    }
}

#[derive(Debug, Digital)]
pub struct I<const DATA: usize, const ADDR: usize> {
    pub axi: MOSI<DATA, ADDR>,
}

#[derive(Debug, Digital)]
pub struct O<const BANK_SIZE: usize, const REG_WIDTH: usize, const DATA: usize> {
    pub axi: MISO<DATA>,
    pub read_data: [Bits<REG_WIDTH>; BANK_SIZE],
}

impl<const BANK_SIZE: usize, const REG_WIDTH: usize, const DATA: usize, const ADDR: usize>
    SynchronousIO for U<BANK_SIZE, REG_WIDTH, DATA, ADDR>
{
    type I = I<DATA, ADDR>;
    type O = O<BANK_SIZE, REG_WIDTH, DATA>;
    type Kernel = bank_kernel<BANK_SIZE, REG_WIDTH, DATA, ADDR>;
}

#[kernel]
pub fn bank_kernel<
    const BANK_SIZE: usize,
    const REG_WIDTH: usize,
    const DATA: usize,
    const ADDR: usize,
>(
    _cr: ClockReset,
    i: I<DATA, ADDR>,
    q: Q<BANK_SIZE, REG_WIDTH, DATA, ADDR>,
) -> (
    O<BANK_SIZE, REG_WIDTH, DATA>,
    D<BANK_SIZE, REG_WIDTH, DATA, ADDR>,
) {
    let mut d = D::<BANK_SIZE, REG_WIDTH, DATA, ADDR>::dont_care();
    let mut o = O::<BANK_SIZE, REG_WIDTH, DATA>::dont_care();
    // Connect the read bridge inputs and outputs to the bus
    d.read_bridge.axi = i.axi.read;
    o.axi.read = q.read_bridge.axi;
    // Connect the write bridge inputs and outputs to the bus
    d.write_bridge.axi = i.axi.write;
    o.axi.write = q.write_bridge.axi;
    // Connect the registers
    for i in 0..BANK_SIZE {
        d.reg[i] = q.reg[i];
    }
    let max_bank: Bits<ADDR> = bits(BANK_SIZE as u128);
    trace("max_bank", &max_bank);
    // Determine if a read was requested
    let (read_requested, read_addr) = unpack::<Bits<ADDR>>(q.read_bridge.cmd);
    // We can only accept new read commands if the reply sender is not full
    d.read_bridge.cmd_next = false;
    d.read_bridge.reply = None;
    if !q.read_bridge.reply_full && read_requested {
        // Acq the command
        d.read_bridge.cmd_next = true;
        let word_addr = read_addr >> q.right_shift;
        trace("word_addr", &word_addr);
        if word_addr < max_bank {
            d.read_bridge.reply = Some(Ok(q.reg[word_addr].resize()));
        } else {
            d.read_bridge.reply = Some(Err(AXI4Error::DECERR));
        }
    }
    // Determine if a write was requested
    let (write_requested, (write_addr, write_data)) =
        unpack::<(Bits<ADDR>, Bits<DATA>)>(q.write_bridge.cmd);
    // We can only accept new write commands if the reply sender is not full
    d.write_bridge.cmd_next = false;
    d.write_bridge.reply = None;
    if !q.write_bridge.reply_full && write_requested {
        // Ack the command
        d.write_bridge.cmd_next = true;
        let word_addr = write_addr >> q.right_shift;
        if word_addr < max_bank {
            d.reg[word_addr] = write_data.resize();
            d.write_bridge.reply = Some(Ok(()));
        } else {
            d.write_bridge.reply = Some(Err(AXI4Error::DECERR));
        }
    }
    // Copy out the register
    o.read_data = q.reg;
    (o, d)
}
