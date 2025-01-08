use rhdl::prelude::*;

use crate::{
    axi4lite::{
        basic::bridge,
        types::{strobe_to_mask, AXI4Error, AxilAddr, AxilData, WriteCommand, MISO, MOSI},
    },
    core::{dff, option::unpack},
};

// Each register is at a different word address

#[derive(Clone, Debug, SynchronousDQ, Synchronous)]
pub struct U<const BANK_SIZE: usize> {
    // We need a read bridge
    read_bridge: bridge::read::U,
    // And a set of registers to hold the values
    reg: [dff::U<AxilData>; BANK_SIZE],
    // And a write bridge
    write_bridge: bridge::write::U,
}

impl<const BANK_SIZE: usize> Default for U<BANK_SIZE> {
    fn default() -> Self {
        Self {
            read_bridge: Default::default(),
            reg: array_init::array_init(|_| Default::default()),
            write_bridge: Default::default(),
        }
    }
}

#[derive(PartialEq, Debug, Digital)]
pub struct I {
    pub axi: MOSI,
}

#[derive(PartialEq, Debug, Digital)]
pub struct O<const BANK_SIZE: usize> {
    pub axi: MISO,
    pub read_data: [AxilData; BANK_SIZE],
}

impl<const BANK_SIZE: usize> SynchronousIO for U<BANK_SIZE> {
    type I = I;
    type O = O<BANK_SIZE>;
    type Kernel = bank_kernel<BANK_SIZE>;
}

#[kernel]
pub fn bank_kernel<const BANK_SIZE: usize>(
    _cr: ClockReset,
    i: I,
    q: Q<BANK_SIZE>,
) -> (O<BANK_SIZE>, D<BANK_SIZE>) {
    let mut d = D::<BANK_SIZE>::dont_care();
    let mut o = O::<BANK_SIZE>::dont_care();
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
    let max_bank: AxilAddr = bits(BANK_SIZE as u128);
    trace("max_bank", &max_bank);
    // Determine if a read was requested
    let (read_requested, read_addr) = unpack::<AxilAddr>(q.read_bridge.cmd);
    // We can only accept new read commands if the reply sender is not full
    d.read_bridge.cmd_next = false;
    d.read_bridge.reply = None;
    if !q.read_bridge.reply_full && read_requested {
        // Acq the command
        d.read_bridge.cmd_next = true;
        // The right shift 2 here is because the address is in 8-bit bytes (octets),
        // and we want to address 32 bit words.  Each work is 4 bytes.  Thus we
        // need to divide the byte address by 4 to get the word address.
        let word_addr = read_addr >> 2;
        trace("word_addr", &word_addr);
        if word_addr < max_bank {
            d.read_bridge.reply = Some(Ok(q.reg[word_addr].resize()));
        } else {
            d.read_bridge.reply = Some(Err(AXI4Error::DECERR));
        }
    }
    // Determine if a write was requested
    let (write_requested, write_cmd) = unpack::<WriteCommand>(q.write_bridge.cmd);
    // We can only accept new write commands if the reply sender is not full
    d.write_bridge.cmd_next = false;
    d.write_bridge.reply = None;
    if !q.write_bridge.reply_full && write_requested {
        // Ack the command
        d.write_bridge.cmd_next = true;
        let word_addr = write_cmd.addr >> 2;
        if word_addr < max_bank {
            let mask = strobe_to_mask(write_cmd.strobed_data.strobe);
            d.reg[word_addr] = (write_cmd.strobed_data.data & mask) | (q.reg[word_addr] & !mask);
            d.write_bridge.reply = Some(Ok(()));
        } else {
            d.write_bridge.reply = Some(Err(AXI4Error::DECERR));
        }
    }
    // Copy out the register
    o.read_data = q.reg;
    (o, d)
}
