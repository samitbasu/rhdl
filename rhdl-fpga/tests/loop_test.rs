use rhdl::{core::ntl::builder::build_btl_from_rtl, prelude::*};
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
    //    SimpleLogger::init(log::LevelFilter::Debug, simplelog::Config::default()).unwrap();
    //let switch: ReadSwitch<2> = ReadSwitch::try_new::<decode_addr>()?;
    let obj = compile_design::<kernel<2>>(CompilationMode::Synchronous)?;
    let mut file = std::fs::File::create("loop.rtl").unwrap();
    write!(file, "{:?}", obj).unwrap();
    let btl = build_btl_from_rtl(&obj);
    let mut file = std::fs::File::create("loop.btl").unwrap();
    write!(file, "{:?}", btl).unwrap();
    //    switch.yosys_check()?;
    //    drc::no_combinatorial_paths(&switch)?;
    Ok(())
}
