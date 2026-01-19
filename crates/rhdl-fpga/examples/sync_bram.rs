use rhdl::prelude::*;
use rhdl_fpga::{
    core::ram::synchronous::{In, SyncBRAM, Write},
    doc::write_svg_as_markdown,
};

fn main() -> Result<(), RHDLError> {
    let read = [b4(1), b4(2), b4(1), b4(1)];
    let no_write = Write {
        addr: b4(0),
        value: b8(0),
        enable: false,
    };
    let write = [
        no_write,
        Write {
            addr: b4(1),
            value: b8(42),
            enable: true,
        },
        no_write,
        no_write,
    ];
    let input = read
        .into_iter()
        .zip(write)
        .map(|(r, w)| In {
            read_addr: r,
            write: w,
        })
        .with_reset(1)
        .clock_pos_edge(100);
    let uut = SyncBRAM::new((0..).map(|x| (b4(x), b8(x))));
    let vcd = uut.run(input).collect::<SvgFile>();
    let options = SvgOptions::default();
    write_svg_as_markdown(vcd, "sync_bram.md", options)?;
    Ok(())
}
