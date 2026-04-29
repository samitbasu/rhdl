use rhdl::prelude::*;
use rhdl_fpga::{
    doc::write_svg_as_markdown,
    serial_bus::ws2812::{In, Ws2812Driver},
};

fn main() -> Result<(), RHDLError> {
    let mut stream_in: Vec<In> = vec![In {
        pixel: bits(0xA5_5A_3C),
        send: true,
        latch: false,
    }];
    for _ in 0..220 {
        stream_in.push(In {
            pixel: bits(0),
            send: false,
            latch: false,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let uut = Ws2812Driver::<6>::new(bits(2), bits(4), bits(8), bits(16));
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "ws2812.md", options)?;
    Ok(())
}
