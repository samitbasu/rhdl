use rhdl::prelude::*;
use rhdl_fpga::{
    serial_bus::i2c_master::{I2cMaster, In},
    doc::write_svg_as_markdown,
};

fn main() -> Result<(), RHDLError> {
    let mut stream_in: Vec<In> = vec![In {
        addr: bits(0x42),
        data: bits(0x55),
        start: true,
        sda_in: false,
    }];
    for _ in 0..160 {
        stream_in.push(In {
            addr: bits(0),
            data: bits(0),
            start: false,
            sda_in: false,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let uut = I2cMaster::<4>::new(bits(2));
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "i2c_master.md", options)?;
    Ok(())
}
