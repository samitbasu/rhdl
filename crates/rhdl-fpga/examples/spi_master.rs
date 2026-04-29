use rhdl::prelude::*;
use rhdl_fpga::{
    serial_bus::spi_master::{In, SpiMaster},
    doc::write_svg_as_markdown,
};

fn main() -> Result<(), RHDLError> {
    let mut stream_in: Vec<In<8>> = vec![In {
        tx_data: bits(0xA5),
        start: true,
        miso: false,
    }];
    for _ in 0..30 {
        stream_in.push(In {
            tx_data: bits(0),
            start: false,
            miso: false,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let uut = SpiMaster::<8, 4>::default();
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "spi_master.md", options)?;
    Ok(())
}
