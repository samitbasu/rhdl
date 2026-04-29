use rhdl::prelude::*;
use rhdl_fpga::{
    serial_bus::bus_uart::{BusUart, In},
    doc::write_svg_as_markdown,
};

fn main() -> Result<(), RHDLError> {
    let divisor = 6u128;
    let uut = BusUart::<6, 4>::new(bits(divisor));

    // Demo: write 0x55 to DATA, then read STATUS a few cycles later.
    let mut stream_in: Vec<In> = vec![In {
        addr: bits(0),
        write_data: bits(0x55),
        read_enable: false,
        write_enable: true,
        rx: true,
    }];
    for _ in 0..32 {
        stream_in.push(In {
            addr: bits(0),
            write_data: bits(0),
            read_enable: false,
            write_enable: false,
            rx: true,
        });
    }
    stream_in.push(In {
        addr: bits(1),
        write_data: bits(0),
        read_enable: true,
        write_enable: false,
        rx: true,
    });
    for _ in 0..16 {
        stream_in.push(In {
            addr: bits(0),
            write_data: bits(0),
            read_enable: false,
            write_enable: false,
            rx: true,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "bus_uart.md", options)?;
    Ok(())
}
