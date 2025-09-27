use rhdl::prelude::*;
use rhdl_fpga::{
    core::ram::option_sync::{In, OptionSyncBRAM},
    doc::write_svg_as_markdown,
};

fn main() -> Result<(), RHDLError> {
    let read = [b4(4), b4(5), b4(2), b4(1), b4(4), b4(4)];
    let write = [Some((b4(2), b8(42))), None, Some((b4(4), b8(21)))]
        .into_iter()
        .chain(std::iter::repeat(None));
    let inputs = read.into_iter().zip(write).map(|(r, w)| In {
        read_addr: r,
        write: w,
    });
    let inputs = inputs.with_reset(1).clock_pos_edge(100);
    let uut = OptionSyncBRAM::<b8, U4>::new((0..).map(|ndx| (b4(ndx), b8(ndx))));
    let vcd = uut.run(inputs).collect::<Vcd>();
    let options = SvgOptions {
        label_width: 20,
        ..Default::default()
    };
    write_svg_as_markdown(vcd, "option_bram.md", options)?;
    Ok(())
}
