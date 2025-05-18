use rhdl::prelude::*;
use rhdl_fpga::{
    core::ram::{
        asynchronous::ReadI,
        option_async::{In, OptionAsyncBRAM, WriteI},
    },
    doc::write_svg_as_markdown,
};

fn main() -> Result<(), RHDLError> {
    let read = [b3(1), b3(2), b3(1), b3(1)];
    let write = [None, Some((b3(1), b8(42))), None, None, None, None];
    let read = read
        .into_iter()
        .without_reset()
        .clock_pos_edge(100)
        .map(|t| {
            t.map(|(cr, val)| ReadI {
                addr: val,
                clock: cr.clock,
            })
        });
    let write = write
        .into_iter()
        .without_reset()
        .clock_pos_edge(79)
        .map(|t| {
            t.map(|(cr, val)| WriteI {
                data: val,
                clock: cr.clock,
            })
        });
    // Merge them
    let input = merge(read, write, |r, w| In {
        read: signal(r),
        write: signal(w),
    });
    let uut: OptionAsyncBRAM<b8, Red, Blue, U3> =
        OptionAsyncBRAM::new((0..).map(|x| (b3(x), b8(x))));
    let vcd = uut.run(input)?.collect::<Vcd>();
    let options = SvgOptions {
        label_width: 20,
        ..Default::default()
    };
    write_svg_as_markdown(vcd, "option_async_bram.md", options)?;
    Ok(())
}
