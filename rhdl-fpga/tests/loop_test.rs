use rhdl::prelude::*;
use rhdl_fpga::axi4lite::{
    core::switch::read::{Command, ReadSwitch},
    types::{AXI4Error, AxilAddr},
};
use simplelog::SimpleLogger;

const ROM0_BASE: AxilAddr = bits(0x4_000_000);
const ROM1_BASE: AxilAddr = bits(0x6_000_000);

// The decode function
#[kernel]
pub fn decode_addr(_cr: ClockReset, req: AxilAddr) -> Command {
    let rom_0_active = req & ROM0_BASE == ROM0_BASE;
    let rom_1_active = req & ROM1_BASE == ROM1_BASE;
    match (rom_0_active, rom_1_active) {
        (true, false) => Ok((bits(0), req)),
        (true, true) => Ok((bits(1), req)),
        _ => Err(AXI4Error::DECERR),
    }
}

use rhdl_fpga::axi4lite::core::switch::read::kernel;

#[test]
fn test_loop_test() -> miette::Result<()> {
    use std::io::Write;
    let obj = compile_design::<kernel<2>>(CompilationMode::Synchronous)?;
    //    drc::no_combinatorial_paths(&switch)?;
    Ok(())
}
