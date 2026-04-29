use rhdl::prelude::*;
use rhdl_fpga::{
    doc::write_svg_as_markdown,
    serial_bus::smbus_host::{In, SmbusHost},
};

fn main() -> Result<(), RHDLError> {
    let uut = SmbusHost::<4, 16>::new(bits(4), bits(10_000));
    let mut stream_in: Vec<In> = vec![In {
        addr: bits(0x50),
        data: bits(0xA5),
        start: true,
        sda_in: true,
    }];
    for _ in 0..400 {
        stream_in.push(In {
            addr: bits(0),
            data: bits(0),
            start: false,
            sda_in: true,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "smbus_host.md", options)?;
    Ok(())
}
