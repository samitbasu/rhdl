use rhdl::prelude::*;

use crate::{
    axi4lite::{
        basic::bridge,
        types::{AXI4Error, MISO, MOSI},
    },
    core::dff,
};

// Each register is at a different word address

#[derive(Clone, Debug, SynchronousDQ)] //, SynchronousDQ)]
pub struct U<
    const BANK_SIZE: usize = 8,
    const REG_WIDTH: usize = 32,
    const DATA: usize = 32,
    const ADDR: usize = 32,
> {
    // We need a read bridge
    read_bridge: bridge::read::U<DATA, ADDR>,
    // And a set of registers to hold the values
    reg: [dff::U<Bits<REG_WIDTH>>; BANK_SIZE],
    // And a write bridge
    write_bridge: bridge::write::U<DATA, ADDR>,
}

impl<const BANK_SIZE: usize, const REG_WIDTH: usize, const DATA: usize, const ADDR: usize> Default
    for U<BANK_SIZE, REG_WIDTH, DATA, ADDR>
{
    fn default() -> Self {
        Self {
            read_bridge: Default::default(),
            reg: array_init::array_init(|_| Default::default()),
            write_bridge: Default::default(),
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
    // Do not stop the write bridge
    d.write_bridge.full = false;
    o.axi.write = q.write_bridge.axi;
    // Connect the registers
    for i in 0..BANK_SIZE {
        d.reg[i] = q.reg[i];
    }
    // The addresses are in bytes.  To get a word
    // address, we need to compute the number of bytes
    // per word.
    let bytes_per_word: Bits<8> = bits(({ DATA } >> 3) as u128);
    let max_bank: Bits<ADDR> = bits(BANK_SIZE as u128);
    // We can then compute a modified address by shifting
    if let Some(read_addr) = q.read_bridge.read {
        let word_addr = read_addr >> bytes_per_word;
        if word_addr < max_bank {
            d.read_bridge.data = Ok(q.reg[word_addr].resize());
        } else {
            d.read_bridge.data = Err(AXI4Error::DECERR);
        }
    }
    if let Some((write_addr, value)) = q.write_bridge.write {
        let word_addr = write_addr >> bytes_per_word;
        if word_addr < max_bank {
            d.reg[word_addr] = value.resize();
        }
    }
    o.read_data = q.reg;
    (o, d)
}
