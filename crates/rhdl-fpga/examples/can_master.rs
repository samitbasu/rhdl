use rhdl::prelude::*;
use rhdl_fpga::{
    serial_bus::can_master::{CanMaster, In},
    doc::write_svg_as_markdown,
};

fn main() -> Result<(), RHDLError> {
    let uut = CanMaster::<5>::new(bits(4));
    let mut stream_in: Vec<In> = vec![In {
        id: bits(0x123),
        dlc: bits(1),
        data: bits(0xA5_00_00_00_00_00_00_00),
        start: true,
    }];
    for _ in 0..400 {
        stream_in.push(In {
            id: bits(0),
            dlc: bits(0),
            data: bits(0),
            start: false,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "can_master.md", options)?;
    Ok(())
}
