use rhdl::prelude::*;
use rhdl_fpga::{
    core::ram::asynchronous::{AsyncBRAM, In, ReadI, WriteI},
    doc::write_svg_as_markdown,
};

fn main() -> Result<(), RHDLError> {
    let read = [b3(1), b3(2), b3(1), b3(1)];
    let write = [None, Some((b3(1), 42)), None, None, None, None];
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
            t.map(|(cr, val)| match val {
                Some((addr, data)) => WriteI {
                    addr,
                    data: bits(data),
                    enable: true,
                    clock: cr.clock,
                },
                None => WriteI {
                    addr: b3(0),
                    data: b8(0),
                    enable: false,
                    clock: cr.clock,
                },
            })
        });
    // Merge them
    let input = merge(read, write, |r, w| In {
        read: signal(r),
        write: signal(w),
    });
    let uut: AsyncBRAM<b8, Red, Blue, 3> = AsyncBRAM::new((0..).map(|x| (b3(x), b8(x))));
    let vcd = uut.run(input).collect::<Vcd>();
    let options = SvgOptions {
        label_width: 20,
        ..Default::default()
    };
    write_svg_as_markdown(vcd, "async_bram.md", options)?;
    Ok(())
}
