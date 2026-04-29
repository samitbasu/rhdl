use rhdl::prelude::*;
use rhdl_fpga::{
    doc::write_svg_as_markdown,
    serial_bus::uart_tx::{In, UartTx},
};

fn main() -> Result<(), RHDLError> {
    // Send 0xA5, then 0x42, with a small idle gap between.
    let mut stream_in: Vec<In> = Vec::new();
    stream_in.push(In {
        data: bits(0xA5),
        send: true,
    });
    for _ in 0..50 {
        stream_in.push(In {
            data: bits(0),
            send: false,
        });
    }
    stream_in.push(In {
        data: bits(0x42),
        send: true,
    });
    for _ in 0..50 {
        stream_in.push(In {
            data: bits(0),
            send: false,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let uut = UartTx::<6>::new(bits(4));
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "uart_tx.md", options)?;
    Ok(())
}
